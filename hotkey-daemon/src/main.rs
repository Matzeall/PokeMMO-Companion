#[cfg(unix)]
mod main_unix;
#[cfg(unix)]
use crate::main_unix::platform;

#[cfg(windows)]
mod main_win;
#[cfg(windows)]
use crate::main_win::platform;

fn main() {
    platform::main();
}
