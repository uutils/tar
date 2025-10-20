use crate::options::TarOptions;
use crate::archive::CreateOperation;
use clap::ArgMatches;
use uucore::error::UResult;
use crate::list::*;

/// Selects the operation that tar will perform with this execution
/// of tar
pub enum Operation {
    Concatenate,
    Create,
    Diff,
    List,
    Append,
    Update,
    Extract
}

impl TarOperation for Operation {
    fn exec(&self, options: &TarOptions) -> UResult<()> {
        match self {
            Self::List => ListOperation.exec(options),
            Self::Create => CreateOperation.exec(options),
            _ => ListOperation.exec(options)
            // Self::Diff => d.exec(options),
            // Self::Append => a.exec(options),
            // Self::Update => u.exec(options),
            // Self::Extract => e.exec(options),
            // Self::Concatenate => c.exec(options),
        }
    }
}


/// Trait to define execution of selected operation determined by the
/// command line arguments during tar execution
pub trait TarOperation {
    fn exec(&self, options: &TarOptions) -> UResult<()>;
}
