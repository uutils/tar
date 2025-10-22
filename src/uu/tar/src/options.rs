use crate::operation::Operation;
use crate::TarError;
use clap::{ArgMatches, Id};
use std::path::PathBuf;
use uucore::error::UResult;

/// [`TarOptions`] Holds common information that is parsed from 
/// command line arguments. That changes the current execution of
/// tar.
// TODO: Come up with a different name
// other than TarOptions...with Options.
// Maybe TarParams?
#[allow(dead_code)]
pub struct TarOptions {
    block_size: usize,
    files: Vec<PathBuf>,
    options: Vec<TarOption>,
}

/// [`Default`] Produces safe default values for options
/// for this tar execution. Block-Size of 512 bytes, Empty vec's of 
/// options and file names.
impl Default for TarOptions {
    fn default() -> TarOptions {
        Self {
            block_size: 512,
            options: Vec::new(),
            files: Vec::new(),
        }
    }
}


// NOTE: I feel like this is just reimplmenting the parsing functionality of
// clap
impl From<&ArgMatches> for TarOptions {
    fn from(matches: &ArgMatches) -> TarOptions {
        let mut fp = vec![];
        let mut ops = Self::default();
        if let Ok(opts_option) = matches.try_get_many::<Id>("options") {
            if let Some(opts_id) = opts_option {
                for opt_id in opts_id {
                    match opt_id.as_str() {
                        "verbose" => {
                            ops.options_mut().push(TarOption::Verbose);
                        }
                        "file" | "files" => {
                            if let Some(files) = matches.get_many::<PathBuf>(opt_id.as_str()) {
                                for file in files {
                                    fp.push(file.to_owned());
                                }
                            }
                            ops.files_mut().append(&mut fp);
                        }
                        _ => {}
                    }
                }
            }
        }
        ops
    }
}

impl TarOptions {
    /// Convence method that parses the [`ArgMatches`] 
    /// processed by clap into [`TarOptions`] and selects
    /// the appropriate [`Operation`] for execution given back to the caller in a
    /// tuple of ([`Operation`], [`TarOptions`])
    pub fn with_operation(matches: &ArgMatches) -> UResult<(Operation, Self)> {
        let options = Self::from(matches);
        if let Ok(operation) = matches.try_get_one::<Id>("operations") {
            if let Some(o) = operation {
                Ok((Operation::try_from(o)?, options))
            } else {
                Err(Box::new(TarError::NotGood))
            }
        } else {
            Err(Box::new(TarError::NotGood))
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

/// [`TarOption`] Enum of avaliable tar options for later use
/// by [`Operation`] impls, eg. List, Create, Delete
#[allow(dead_code)]
pub enum TarOption {
    AbsoluteNames,
    ACLs,
    AfterDate,
    Anchored,
    AtimePreserve { arg: String },
    Verbose,
}
