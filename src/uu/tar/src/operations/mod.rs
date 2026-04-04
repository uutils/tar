// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(test)]
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

pub mod compression;
pub mod create;
pub mod extract;
pub mod list;

#[cfg(test)]
pub(crate) fn test_cwd_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
pub(crate) struct TestDirGuard {
    old_dir: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

#[cfg(test)]
impl TestDirGuard {
    pub(crate) fn enter(path: &Path) -> Self {
        let guard = test_cwd_lock().lock().unwrap();
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();
        Self {
            old_dir,
            _guard: guard,
        }
    }
}

#[cfg(test)]
impl Drop for TestDirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.old_dir);
    }
}
