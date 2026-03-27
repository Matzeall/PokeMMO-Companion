use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use tokio::runtime::{Builder, Handle};
use tokio::sync::oneshot::{self, Sender};

pub struct AsyncManager {
    // handle to schedule async tasks from sync code
    pub runtime_handle: Handle,

    shutdown_tx: Option<Sender<()>>, // Option, because sending it consumes it -> null
    thread_handle: Option<JoinHandle<()>>, // Option, because joining consumes it -> null
}

impl AsyncManager {
    pub fn new() -> AsyncManager {
        // Create the runtime (multi-threaded worker pool)
        let runtime = Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .expect("failed to build runtime");

        let runtime_handle = runtime.handle().clone();

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let thread_handle = thread::spawn(move || {
            // block on new thread -> schedule all async tasks here
            runtime.block_on(async move {
                // Wait until the main thread signals shutdown.
                let _ = shutdown_rx.await;
            });
        });

        Self {
            runtime_handle,
            shutdown_tx: Some(shutdown_tx),
            thread_handle: Some(thread_handle),
        }
    }

    pub fn spawn_with_response<T, E, F>(&self, fut: F) -> Receiver<Result<T, E>>
    where
        T: Send + 'static,
        E: Send + 'static,
        F: std::future::Future<Output = Result<T, E>> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<Result<T, E>>();

        self.runtime_handle.spawn(async move {
            let res = fut.await;
            let _ = tx.send(res);
        });

        rx
    }

    pub fn shutdown(&mut self) {
        // send shutdown if present (ignore error if receiver already gone)
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        // join thread (ignore panic info)
        if let Some(jh) = self.thread_handle.take() {
            let _ = jh.join();
        }
    }
}

impl Drop for AsyncManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
