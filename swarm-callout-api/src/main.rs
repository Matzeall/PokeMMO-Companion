use tokio::{runtime::Builder, task::JoinHandle};

use crate::{
    data_source::{DataSource, SwarmData, discord_bot::DiscordBot},
    data_target::DataTarget,
};
use std::{thread, time::Duration};

pub mod data_source;
pub mod data_target;

const UPDATE_INTERVAL: std::time::Duration = Duration::from_mins(5);

struct DataSourceRuntime {
    source: Box<dyn DataSource>,
    worker: Option<JoinHandle<()>>,
}

impl DataSourceRuntime {
    fn is_data_source_active(&self) -> bool {
        self.worker.is_some() && self.source.is_reliable()
    }
}

struct CalloutApiUpdater {
    // the order in the data_sources vec determines priority
    // and reliability from best to worst(fallback)
    data_sources: Vec<DataSourceRuntime>,

    // anytime a new state is determined from a data_source all target are updated
    data_target: Vec<Box<dyn DataTarget>>,

    tokio_rt_handle: tokio::runtime::Handle,

    current_swarm_state: SwarmData,
}

impl CalloutApiUpdater {
    fn spawn_all(&mut self) {
        for data_source in &mut self.data_sources {
            if let Some(handle) = &data_source.worker {
                handle.abort();
            }

            data_source.worker = Some(data_source.source.spawn(self.tokio_rt_handle.clone()));
        }
    }

    fn fetch_current_swarm_state(&mut self) -> SwarmData {
        for i in 0..self.data_sources.len() {
            let data_source = &mut self.data_sources[i];
            if data_source.is_data_source_active() {
                return data_source.source.get_current_data();
            }
        }
        SwarmData::default()
    }

    fn publish_on_all_targets(&self, new_swarm_state: &SwarmData) {
        for target in &self.data_target {
            target.publish_data(new_swarm_state);
        }
    }
}

fn main() {
    dotenvy::dotenv().expect("no .env file found -> aborting");

    let tokio_runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let discord_bot = DataSourceRuntime {
        source: Box::new(DiscordBot::new()),
        worker: None,
    };

    let mut api_updater = CalloutApiUpdater {
        data_sources: vec![discord_bot],
        data_target: Vec::new(),
        tokio_rt_handle: tokio_runtime.handle().clone(),
        current_swarm_state: SwarmData::default(),
    };

    api_updater.spawn_all();

    // initial update
    let initial_state = api_updater.fetch_current_swarm_state();
    api_updater.publish_on_all_targets(&initial_state);
    thread::sleep(UPDATE_INTERVAL);

    // update all targets every 5 min (UPDATE_INTERVAL)
    loop {
        let new_swarm_state = api_updater.fetch_current_swarm_state();
        println!("Checking SwarmState ...");

        if new_swarm_state != api_updater.current_swarm_state {
            println!("New Swarm State detected: {:?}", new_swarm_state);
            api_updater.publish_on_all_targets(&new_swarm_state);
            api_updater.current_swarm_state = new_swarm_state;
        }

        thread::sleep(UPDATE_INTERVAL);
    }
}
