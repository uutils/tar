use crate::errors::TarError;
use crate::operations::OperationKind;
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
    archive: PathBuf,
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
            archive: PathBuf::default(),
            options: Vec::new(),
            files: Vec::new(),
        }
    }
}

// NOTE: I feel like this is just reimplmenting the parsing functionality of
// clap
impl From<&ArgMatches> for TarOptions {
    fn from(matches: &ArgMatches) -> TarOptions {
        println!("{:?}", matches);
        let mut fp = vec![];
        let mut ops = Self::default();
        for i in matches.ids() {
            match i.as_str() {
                "verbose" => {
                    ops.options_mut().push(TarOption::Verbose);
                }
                "files" => {
                    if let Some(files) = matches.get_many::<PathBuf>(i.as_str()) {
                        for file in files {
                            fp.push(file.to_owned());
                        }
                    }
                    ops.files_mut().append(&mut fp);
                }
                "archive" => {
                    if let Some(a) = matches.get_one::<PathBuf>(i.as_str()) {
                        ops.archive = a.to_owned();
                    }
                }
                _ => {}
            }
        }
        ops
    }
}

impl TarOptions {
    /// Convence method that parses the [`ArgMatches`]
    /// processed by clap into [`TarOptions`] and selects
    /// the appropriate [`OperationKind`] for execution given back to the caller in a
    /// tuple of ([`OperationKind`], [`TarOptions`])
    pub fn with_operation(matches: &ArgMatches) -> UResult<(OperationKind, Self)> {
        if let Some((o, m)) = matches.subcommand() {
            Ok((OperationKind::try_from(o)?, Self::from(m)))
        } else {
            // TODO: update messaging
            Err(Box::new(TarError::TarOperationError(
                "Error processing: Operations".to_string(),
            )))
        }
    }
}

#[allow(dead_code)]
impl TarOptions {
    pub fn files(&self) -> &Vec<PathBuf> {
        &self.files
    }
    pub fn files_mut(&mut self) -> &mut Vec<PathBuf> {
        &mut self.files
    }
    pub fn archive(&self) -> &PathBuf {
        &self.archive
    }
    pub fn archive_mut(&mut self) -> &mut PathBuf {
        &mut self.archive
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
