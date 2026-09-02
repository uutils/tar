// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

pub mod create;
pub mod extract;
pub mod list;

/// Strip `n` leading path components from `path`.
/// Returns `None` if all components are consumed (entry should be skipped).
pub fn strip_leading_components(path: &Path, n: u32) -> Option<PathBuf> {
    if n == 0 {
        return Some(path.to_path_buf());
    }
    let mut components = path.components();
    for _ in 0..n {
        components.next()?;
    }
    let remaining: PathBuf = components.collect();
    if remaining.as_os_str().is_empty() {
        None
    } else {
        Some(remaining)
    }
}

/// Match `text` against a shell glob `pattern` (`*` = any string, `?` = any char).
pub fn wildcard_match(pattern: &str, text: &str) -> bool {
    let mut regex_str = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => regex_str.push_str(".*"),
            '?' => regex_str.push('.'),
            c => regex_str.push_str(&regex::escape(&c.to_string())),
        }
    }
    regex_str.push('$');
    regex::Regex::new(&regex_str)
        .map(|r| r.is_match(text))
        .unwrap_or(false)
}

/// Check whether an entry path matches the given list of patterns.
/// If `patterns` is empty, all entries match. Without wildcards, a pattern
/// matches entries whose normalised path equals the pattern or whose path
/// is contained in the named directory.
pub fn entry_matches(path: &Path, patterns: &[PathBuf], wildcards: bool) -> bool {
    if patterns.is_empty() {
        return true;
    }
    let path_str = path.to_string_lossy();
    let normalised = path_str.trim_end_matches('/');
    for pattern in patterns {
        let pat = pattern.to_string_lossy();
        if wildcards {
            if wildcard_match(&pat, &path_str) || wildcard_match(&pat, normalised) {
                return true;
            }
        } else {
            let norm_pat = pat.trim_end_matches('/');
            if normalised == norm_pat {
                return true;
            }
            // entry lives inside the named directory
            if path_str.starts_with(&format!("{}/", norm_pat)) {
                return true;
            }
        }
    }
    false
}

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

#[cfg(test)]
mod tests {
    use super::*;

    // strip_leading_components

    #[test]
    fn test_strip_zero_components() {
        let p = PathBuf::from("a/b/c.txt");
        assert_eq!(strip_leading_components(&p, 0), Some(p));
    }

    #[test]
    fn test_strip_one_component() {
        assert_eq!(
            strip_leading_components(Path::new("a/b/c.txt"), 1),
            Some(PathBuf::from("b/c.txt"))
        );
    }

    #[test]
    fn test_strip_all_components_returns_none() {
        assert_eq!(strip_leading_components(Path::new("a/b"), 3), None);
    }

    #[test]
    fn test_strip_exact_components_returns_none() {
        // "a/b" stripped by 2 leaves an empty path
        assert_eq!(strip_leading_components(Path::new("a/b"), 2), None);
    }

    // wildcard_match

    #[test]
    fn test_wildcard_star_matches_any_string() {
        assert!(wildcard_match("*.txt", "file.txt"));
        assert!(wildcard_match("*.txt", "dir/file.txt"));
    }

    #[test]
    fn test_wildcard_question_matches_single_char() {
        assert!(wildcard_match("file?.txt", "file1.txt"));
        assert!(!wildcard_match("file?.txt", "file10.txt"));
    }

    #[test]
    fn test_wildcard_no_match() {
        assert!(!wildcard_match("*.rs", "file.txt"));
    }

    #[test]
    fn test_wildcard_exact_literal() {
        assert!(wildcard_match("exact.txt", "exact.txt"));
        assert!(!wildcard_match("exact.txt", "other.txt"));
    }

    // entry_matches

    #[test]
    fn test_entry_matches_empty_patterns_matches_all() {
        assert!(entry_matches(Path::new("anything.txt"), &[], false));
    }

    #[test]
    fn test_entry_matches_exact() {
        let patterns = vec![PathBuf::from("file.txt")];
        assert!(entry_matches(Path::new("file.txt"), &patterns, false));
        assert!(!entry_matches(Path::new("other.txt"), &patterns, false));
    }

    #[test]
    fn test_entry_matches_directory_prefix() {
        let patterns = vec![PathBuf::from("dir")];
        assert!(entry_matches(Path::new("dir/file.txt"), &patterns, false));
        assert!(!entry_matches(
            Path::new("other/file.txt"),
            &patterns,
            false
        ));
    }

    #[test]
    fn test_entry_matches_wildcard() {
        let patterns = vec![PathBuf::from("*.txt")];
        assert!(entry_matches(Path::new("file.txt"), &patterns, true));
        assert!(!entry_matches(Path::new("file.rs"), &patterns, true));
    }
}
