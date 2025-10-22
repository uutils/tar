use crate::archive::ArchiveList;
use crate::operation::TarOperation;
use crate::options::{TarOption, TarOptions};
use uucore::error::UResult;

pub(crate) struct ListOperation;

impl TarOperation for ListOperation {
    fn exec(&self, options: &TarOptions) -> UResult<()> {
        let list = ArchiveList::try_from(options.files().as_slice())?;
        let verbose = options.options().iter().any(|x| match x {
            TarOption::Verbose => true,
            _ => false,
        });
        for archive in list {
            for member in archive.members() {
                member.print_member(verbose);
            }
        }
        Ok(())
    }
}
