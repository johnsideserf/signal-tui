//! Optional debug logger — writes to signal-tui-debug.log when --debug is passed.

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static ENABLED: AtomicBool = AtomicBool::new(false);
static LOCK: Mutex<()> = Mutex::new(());

pub fn enable() {
    ENABLED.store(true, Ordering::Relaxed);
}

pub fn log(msg: &str) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let _guard = LOCK.lock();
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("signal-tui-debug.log")
    {
        let now = chrono::Local::now().format("%H:%M:%S%.3f");
        let _ = writeln!(f, "[{now}] {msg}");
    }
}

pub fn logf(args: std::fmt::Arguments<'_>) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    log(&format!("{args}"));
}
