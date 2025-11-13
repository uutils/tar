use crate::errors::TarError;
use crate::operations::Create;
use crate::operations::Extract;
use crate::operations::List;
use crate::options::TarOptions;
use clap::Id;
use uucore::error::UResult;

/// The [`OperationKind`] Enum representation of Acdtrux arguments which is
/// later leveraged as selector for enum dispatch by the [`TarOperation`]
/// trait
pub enum OperationKind {
    Concatenate,
    Create,
    Diff,
    List,
    Append,
    Update,
    Extract,
}

impl TryFrom<&str> for OperationKind {
    type Error = TarError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "concate" => Ok(Self::Concatenate),
            "create" => Ok(Self::Create),
            "diff" => Ok(Self::Diff),
            "list" => Ok(Self::List),
            "append" => Ok(Self::Append),
            "update" => Ok(Self::Update),
            "extract" => Ok(Self::Extract),
            _ => Err(
                TarError::TarOperationError(
                    format!("Invalid operation selected: {}", value.to_string())
                )
            ),
        }
    }
}

impl TarOperation for OperationKind {
    fn exec(&self, options: &TarOptions) -> UResult<()> {
        match self {
            Self::List => List.exec(options),
            Self::Create => Create.exec(options),
            Self::Diff => unimplemented!(),
            Self::Append => unimplemented!(),
            Self::Update => unimplemented!(),
            Self::Extract => Extract.exec(options),
            Self::Concatenate => unimplemented!(),
        }
    }
}

/// [`TarOperation`] allows enum dispatch by enforcing the impl of the
/// trait to create the functionality to perform the operation requested via
/// the command line arg for this execution of tar
pub trait TarOperation {
    fn exec(&self, options: &TarOptions) -> UResult<()>;
}
