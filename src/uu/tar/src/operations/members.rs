// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::{report_recoverable, TarError};
use std::path::{Path, PathBuf};
use uucore::error::UResult;

/// The member names given to `-t` or `-x`. Without any, every entry is
/// selected. A name selects the entry with that name and, like GNU tar, the
/// entries under it when it names a directory.
pub struct MemberFilter {
    names: Vec<(PathBuf, bool)>,
}

impl MemberFilter {
    pub fn new(names: &[PathBuf]) -> Self {
        Self {
            names: names.iter().map(|name| (name.clone(), false)).collect(),
        }
    }

    /// Whether the entry at `path` is selected, recording which names matched.
    pub fn selects(&mut self, path: &Path) -> bool {
        if self.names.is_empty() {
            return true;
        }
        let mut selected = false;
        for (name, matched) in &mut self.names {
            if path.starts_with(&*name) {
                *matched = true;
                selected = true;
            }
        }
        selected
    }

    /// Reports each name that selected no entry. Returns true if there was one.
    pub fn report_unmatched(&self) -> UResult<bool> {
        let mut any = false;
        for (name, matched) in &self.names {
            if !matched {
                report_recoverable(&TarError::NotFoundInArchive(name.clone()))?;
                any = true;
            }
        }
        Ok(any)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_names_selects_everything() {
        let mut filter = MemberFilter::new(&[]);
        assert!(filter.selects(Path::new("a/b")));
        assert!(!filter.report_unmatched().unwrap());
    }

    #[test]
    fn a_name_selects_itself_and_what_is_under_it() {
        let mut filter = MemberFilter::new(&[PathBuf::from("src/d")]);
        assert!(filter.selects(Path::new("src/d/")));
        assert!(filter.selects(Path::new("src/d/b.txt")));
        assert!(!filter.selects(Path::new("src/a")));
        // A name matches whole components only.
        assert!(!filter.selects(Path::new("src/dd")));
    }
}
