use crate::archive::CreateOperation;
use crate::list::ListOperation;
use crate::options::TarOptions;
use crate::TarError;
use clap::Id;
use uucore::error::UResult;

/// [`Operation`] Enum representation of Acdtrux arguments which is
/// later leveraged as selector for enum dispatch by the [`TarOperation`]
/// trait
pub enum Operation {
    Concatenate,
    Create,
    Diff,
    List,
    Append,
    Update,
    Extract,
}

impl TryFrom<&Id> for Operation {
    type Error = TarError;
    fn try_from(value: &Id) -> Result<Self, Self::Error> {
        match value.as_str() {
            "concate" => return Ok(Self::Concatenate),
            "create" => return Ok(Self::Create),
            "diff" => return Ok(Self::Diff),
            "list" => return Ok(Self::List),
            "append" => return Ok(Self::Append),
            "update" => return Ok(Self::Update),
            "extract" => return Ok(Self::Extract),
            _ => return Err(TarError::InvalidOperation(value.to_string())),
        }
    }
}

impl TarOperation for Operation {
    fn exec(&self, options: &TarOptions) -> UResult<()> {
        match self {
            Self::List => ListOperation.exec(options),
            Self::Create => CreateOperation.exec(options),
            Self::Diff => unimplemented!(),
            Self::Append => unimplemented!(),
            Self::Update => unimplemented!(),
            Self::Extract => unimplemented!(),
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
