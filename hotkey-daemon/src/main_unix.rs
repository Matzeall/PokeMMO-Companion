// Using root access to fully bypass waylands security model, is the only way to achieve global
// shortcuts, if the XDG_Desktop_Portal of your desktop environment doesn't support GlobalHotkeys
// For Ubuntu and the GNOME DE this is the case for every version below GNOME 48 (->Ubuntu 25.04)

// For testing try to connect to the socket with: " $ socat - UNIX-CONNECT:./hotkeys.sock"

#![cfg(unix)]

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use evdev::{Device, EventSummary, KeyCode};
use nix::unistd::{Gid, Uid, setgid, setuid};
use std::{
    env, fs,
    io::{BufWriter, Write},
    os::unix::{
        fs::PermissionsExt,
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

// parsed cli args
#[derive(Parser, Debug)]
struct CLIOpts {
    /// UID to drop to after open_input_devices
    #[arg(long, value_name = "UID")]
    drop_uid: Option<u32>,

    /// GID to drop to after open_input_devices
    #[arg(long, value_name = "GID")]
    drop_gid: Option<u32>,
}

// messages that will be sent to clients
const FOCUS: &str = "focus";
const CLOSE: &str = "close";
const VISIBLE: &str = "visible";

fn main() -> Result<()> {
    println!("hotkey-daemon: Starting hotkey-deamon ...\n");
    let opts = CLIOpts::parse();

    let socket_path = socket_path()?;

    if socket_path.exists() {
        fs::remove_file(&socket_path)?;
    }

    // while priviledged open input files in /dev/input/event*
    let mut devs = open_input_devices()?;

    drop_privileges(opts)?; // no sudo afterwards, only necessary to open

    // creates the socket(-file)
    let listener = UnixListener::bind(&socket_path)?;

    let socket_permissions = fs::Permissions::from_mode(0o644); // owner: rw, others: r
    fs::set_permissions(&socket_path, socket_permissions)?;

    // will be populated by the thread if connection comes in
    let clients = Arc::new(Mutex::new(Vec::<UnixStream>::new()));

    // limits the scope of the thread environment
    {
        let clients = clients.clone(); // shared pointer +1
        thread::spawn(move || {
            for incoming in listener.incoming() {
                match incoming {
                    Ok(stream) => {
                        println!("hotkey-daemon: Client connected");
                        clients.lock().unwrap().push(stream);
                    }
                    Err(e) => println!("hotkey-daemon: Socket accept error: {}", e),
                }
            }
        });
    }

    println!("\nhotkey-daemon: Start listening for hotkey combinations...");
    // listen on each device and handle alt + f/c/v keycodes
    let mut alt_down = false;
    loop {
        for dev in &mut devs {
            if let Ok(events) = dev.fetch_events() {
                for ev in events {
                    if let EventSummary::Key(_ev, code, value) = ev.destructure() {
                        // only for debugging purposes
                        // println!("hotkey-daemon: Keyevent( code {:?} = {} )", code, value);
                        match (code, value) {
                            (KeyCode::KEY_LEFTALT, 0) | (KeyCode::KEY_RIGHTALT, 0) => {
                                alt_down = false;
                            }
                            (KeyCode::KEY_LEFTALT, _) | (KeyCode::KEY_RIGHTALT, _) => {
                                alt_down = true;
                            }
                            (KeyCode::KEY_F, 1) if alt_down => {
                                println!("hotkey-daemon: Alt+F pressed → notifying clients");
                                notify_clients_of(&clients, FOCUS);
                            }
                            (KeyCode::KEY_C, 1) if alt_down => {
                                println!("hotkey-daemon: Alt+C pressed → notifying clients");
                                notify_clients_of(&clients, CLOSE);
                            }
                            (KeyCode::KEY_V, 1) if alt_down => {
                                println!("hotkey-daemon: Alt+V pressed → notifying clients");
                                notify_clients_of(&clients, VISIBLE);
                            }
                            // everything else doesn't matter to me
                            // This isn't a keylogger after all 0.o
                            _ => {}
                        }
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}

// next to daemon binary
fn socket_path() -> anyhow::Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| anyhow!("Executable has no parent directory"))?;
    Ok(exe_dir.join("hotkeys.sock"))
}

// after opening /dev/input/event* files, drop sudo privileges
fn drop_privileges(opts: CLIOpts) -> Result<()> {
    let uid = match opts.drop_uid {
        Some(uid) => Uid::from_raw(uid),
        None => match env::var("SUDO_UID") {
            Ok(env_value) => match env_value.parse() {
                Ok(id) => Uid::from_raw(id),
                Err(err) => {
                    println!(
                        "hotkey-daemon: Could not parse SUDO_UID: {}. Falling back to UID 1000.",
                        err
                    );
                    Uid::from_raw(1000)
                }
            },
            Err(err) => {
                println!(
                    "hotkey-daemon: SUDO_UID not set: {}. Falling back to UID 1000.",
                    err
                );
                Uid::from_raw(1000)
            }
        },
    };

    let gid = match opts.drop_gid {
        Some(gid) => Gid::from_raw(gid),
        None => match env::var("SUDO_GID") {
            Ok(env_value) => match env_value.parse() {
                Ok(parsed_gid) => Gid::from_raw(parsed_gid),
                Err(err) => {
                    println!(
                        "hotkey-daemon: Could not parse SUDO_GID: {}. Falling back to GID 1000.",
                        err
                    );
                    Gid::from_raw(1000)
                }
            },
            Err(err) => {
                println!(
                    "hotkey-daemon: SUDO_GID not set: {}. Falling back to GID 1000.",
                    err
                );
                Gid::from_raw(1000)
            }
        },
    };
    setgid(gid).map_err(|e| anyhow!("setgid: {}", e))?;
    setuid(uid).map_err(|e| anyhow!("setuid: {}", e))?;

    println!("hotkey-daemon : dropped privileges to : ({uid}, {gid})");
    Ok(())
}

fn open_input_devices() -> Result<Vec<Device>> {
    let mut devs = Vec::new();
    for entry in fs::read_dir("/dev/input")? {
        let path = entry?.path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("event"))
        {
            match Device::open(&path) {
                Ok(dev) => {
                    // only add keyboards, i.e. has more than one keycode associated
                    if dev
                        .supported_keys()
                        .is_some_and(|keys| (keys.iter().count() != 0))
                    {
                        println!("hotkey-daemon: Opened {:?}", path);
                        // otherwise calls to fetch event block all other devices
                        dev.set_nonblocking(true)?;

                        devs.push(dev);
                    }
                }
                Err(e) => println!("hotkey-daemon: Failed to open {:?}: {}", path, e),
            }
        }
    }
    if devs.is_empty() {
        bail!("No /dev/input/event* devices available");
    }
    Ok(devs)
}

fn notify_clients_of(clients: &Arc<Mutex<Vec<UnixStream>>>, message: &str) {
    let mut guard = clients.lock().unwrap();
    guard.retain_mut(|client| {
        let mut client_receive_buffer = BufWriter::new(client);
        // writes into clients stream -> notifying them
        match writeln!(client_receive_buffer, "{message}") {
            Ok(_) => true,
            Err(e) => {
                println!("hotkey-daemon: Client write failed: {}", e);
                false // error, remove this client from listeners
            }
        }
    });
}
