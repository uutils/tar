use crate::archive::Archive;
use crate::operation::TarOperation;
use crate::options::{TarOption, TarOptions};
use uucore::error::UResult;

pub(crate) struct ListOperation;

impl TarOperation for ListOperation {
    fn exec(&self, options: &TarOptions) -> UResult<()> {
        // TODO: I think there is some sort of option to list a
        // particular member
        let archive = Archive::try_from(options.archive())?;
        let verbose = options.options().iter().any(|x| matches!(x, TarOption::Verbose));
        for member in archive.members() {
            member.print_member(verbose)?;
        }
        Ok(())
    }
}
