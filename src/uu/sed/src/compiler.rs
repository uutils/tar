// Compile the scripts into the internal representation of commands
//
// This file is part of the uutils sed package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::command::{Command, ScriptValue};
use uucore::error::UResult;

pub fn compile(_scripts: Vec<ScriptValue>) -> UResult<Option<Command>> {
    // TODO
    Ok(None)
}
