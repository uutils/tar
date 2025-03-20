// Process the files with the compiled scripts
//
// This file is part of the uutils sed package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::command::Command;
use std::path::PathBuf;
use uucore::error::UResult;

pub fn process(_code: Option<Command>, _files: Vec<PathBuf>) -> UResult<()> {
    // TODO
    Ok(())
}
