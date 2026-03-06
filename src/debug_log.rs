//! Optional debug logger — writes to siggy-debug.log when --debug is passed.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static ENABLED: AtomicBool = AtomicBool::new(false);
static FILE: Mutex<Option<File>> = Mutex::new(None);

pub fn enable() {
    ENABLED.store(true, Ordering::Relaxed);
    let path = "siggy-debug.log";
    if let Ok(f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        // Restrict log file to owner-only access (contains message content)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }
        if let Ok(mut guard) = FILE.lock() {
            *guard = Some(f);
        }
    }
}

pub fn log(msg: &str) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    if let Ok(mut guard) = FILE.lock() {
        if let Some(ref mut f) = *guard {
            let now = chrono::Local::now().format("%H:%M:%S%.3f");
            let _ = writeln!(f, "[{now}] {msg}");
        }
    }
}

pub fn logf(args: std::fmt::Arguments<'_>) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    log(&format!("{args}"));
}
