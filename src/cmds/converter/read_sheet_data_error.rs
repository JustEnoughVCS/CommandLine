use just_enough_vcs::system::sheet_system::sheet::error::ReadSheetDataError;

use crate::systems::cmd::errors::CmdExecuteError;

pub struct ReadSheetDataErrorConverter;

impl ReadSheetDataErrorConverter {
    pub fn to_exec_error(err: ReadSheetDataError) -> CmdExecuteError {
        match err {
            ReadSheetDataError::IOErr(error) => CmdExecuteError::Io(error),
        }
    }
}
