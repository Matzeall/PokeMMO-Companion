#[cfg(unix)]
include!("main_unix.rs");

#[cfg(not(unix))]
fn main() {
    eprintln!("hotkey-daemon only runs on unix-systems");
}
