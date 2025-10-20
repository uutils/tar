use clap::ArgMatches;
use uucore::error::UResult;
use crate::operation::{TarOperation, Operation};
use crate::archive::CreateOperation;
use std::path::PathBuf;
use crate::list::ListOperation;

/// Contains quick access to options given through the
/// command line when calling tar
// TODO: please come up with a different name
// other than TarOptions...with Options
pub struct TarOptions {
    block_size: usize,
    files: Vec<PathBuf>,
    options: Vec<TarOption>
}

impl Default for TarOptions {
    fn default() -> TarOptions {
        Self { block_size: 512, options: Vec::new(), files: Vec::new() }
    }
}

impl From<&ArgMatches> for TarOptions {
    fn from(matches: &ArgMatches) -> TarOptions {
        let mut fp = vec![];
        let mut ops = Self::default();
        if matches.get_flag("verbose") {
            ops.options_mut().push(TarOption::Verbose);
        }
        if matches.get_flag("list") { 
            if let Some(file) = matches.get_one::<PathBuf>("file") {
                fp.push(file.to_owned());
            }
            if let Some(files) = matches.get_many::<PathBuf>("files") {
                for file in files {
                    fp.push(file.to_owned());
                }
            }
        };
        ops.files_mut().append(&mut fp);
        ops
    }
}

impl TarOptions {
    pub fn with_operation(matches: &ArgMatches) -> (Operation, Self) {
        let options = Self::from(matches);
        // default op
        let mut m = vec![];
        for id in matches.ids() {
            if let Some(_) = matches.get_raw_occurrences(id.as_str()) {
                m.push(id.as_str()); 
            } 
        };
        if let Some(arg) = m.iter().next() {
            match *arg {
                "list" => return (Operation::List, options),
                _ => return (Operation::List, options)
            }
        }else{
            return (Operation::Create, options)
        }
    }
}

impl TarOptions {
    pub fn files(&self) -> &Vec<PathBuf> {
        &self.files
    }
    pub fn files_mut(&mut self) -> &mut Vec<PathBuf> {
        &mut self.files
    }
    pub fn options(&self) -> &Vec<TarOption> {
        &self.options
    }
    pub fn options_mut(&mut self) -> &mut Vec<TarOption> {
        &mut self.options
    }
}

/// Options and flags given to tar to augment the operation during calls
/// to the tar via the command line
pub enum TarOption {
    AbsoluteNames,
    ACLs,
    AfterDate,
    Anchored,
    AtimePreserve { arg: String },
    Verbose
}

