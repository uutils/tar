use crate::errors::TarError;
use crate::operations::OperationKind;
use clap::ArgMatches;
use std::path::PathBuf;
use uucore::error::UResult;

/// [`TarParams`] Holds common information that is parsed from
/// command line arguments. That changes the current execution of
/// tar.
#[derive(Default)]
pub struct TarParams {
    archive: PathBuf,
    files: Vec<PathBuf>,
    options: Vec<TarOption>,
}

impl From<&ArgMatches> for TarParams {
    fn from(matches: &ArgMatches) -> TarParams {
        let mut ops = Self::default();

        // -v --verbose
        if matches.get_flag("verbose") {
            ops.options_mut().push(TarOption::Verbose);
        }

        // [FILES]...
        if let Some(files) = matches.get_many::<PathBuf>("files") {
            ops.files = files.map(|x| x.to_owned()).collect();
        }

        // -f --file
        if let Some(a) = matches.get_one::<PathBuf>("file") {
            ops.archive = a.to_owned();
        }

        ops
    }
}

impl TarParams {
    /// Convence method that parses the [`ArgMatches`]
    /// processed by clap into [`TarParams`] and selects
    /// the appropriate [`OperationKind`] for execution given back to the caller in a
    /// tuple of ([`OperationKind`], [`TarParams`])
    pub fn with_operation(matches: &ArgMatches) -> UResult<(OperationKind, Self)> {
        if matches.get_flag("create") {
            Ok((OperationKind::Create, Self::from(matches)))
        } else if matches.get_flag("extract") {
            Ok((OperationKind::Extract, Self::from(matches)))
        } else {
            Err(Box::new(TarError::TarOperationError(format!(
                "Error processing: Unknown or Unimplmented Parameters: {:?}",
                matches
                    .ids()
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
            ))))
        }
    }
}

impl TarParams {
    pub fn files(&self) -> &Vec<PathBuf> {
        &self.files
    }
    pub fn archive(&self) -> &PathBuf {
        &self.archive
    }
    pub fn options(&self) -> &Vec<TarOption> {
        &self.options
    }
    pub fn options_mut(&mut self) -> &mut Vec<TarOption> {
        &mut self.options
    }
}

/// [`TarOption`] Enum of avaliable tar options for later use
/// by [`TarOperation`] impls, eg. List, Create, Delete
pub enum TarOption {
    Verbose,
}
