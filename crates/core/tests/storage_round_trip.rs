//! Verifies the desktop storage backend (file-backed `./catode32-save.json`)
//! survives a write/read round trip.
//!
//! Runs the test inside a temp working directory so a real save file in
//! the repo root isn't clobbered. Single-test file so cargo runs it in
//! its own process. The static `FLASH_STORAGE` cell on firmware has no
//! desktop analogue, but cargo's per-binary process isolation keeps
//! multiple test files from racing on the same `./catode32-save.json`.

#![cfg(feature = "desktop")]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use catode32_core::storage;

/// Serialises the tests in this file. They both flip the process CWD and
/// then read/write `./catode32-save.json`. Running in parallel would
/// have them racing on the same path.
fn lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

struct CwdGuard {
    prev: PathBuf,
    tmp: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl CwdGuard {
    fn enter() -> Self {
        let _lock = lock();
        let prev = env::current_dir().expect("get cwd");
        let mut tmp = env::temp_dir();
        tmp.push(format!(
            "catode32-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&tmp).expect("create tmp");
        env::set_current_dir(&tmp).expect("set cwd");
        Self { prev, tmp, _lock }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.prev);
        let _ = fs::remove_dir_all(&self.tmp);
    }
}

#[test]
fn write_then_read_returns_same_bytes() {
    let _g = CwdGuard::enter();
    storage::init();

    assert!(!storage::has_save(), "tmp dir should start clean");

    let payload = br#"{"hello":"world","n":42}"#;
    assert!(storage::write_next(payload), "write_next should succeed");
    assert!(storage::has_save(), "has_save should report the new file");

    let mut buf = [0u8; 1024];
    let n = storage::read_latest(&mut buf).expect("read_latest");
    assert_eq!(&buf[..n], payload);
}

#[test]
fn erase_removes_save() {
    let _g = CwdGuard::enter();
    storage::init();

    storage::write_next(b"anything");
    assert!(storage::has_save());

    assert!(storage::erase_all());
    assert!(!storage::has_save());

    // Erase on an already-clean slot is a success no-op.
    assert!(storage::erase_all());
}
