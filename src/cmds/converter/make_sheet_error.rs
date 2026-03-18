use crate::systems::cmd::errors::CmdExecuteError;
use just_enough_vcs::system::workspace::workspace::manager::sheet_state::error::MakeSheetError;
use rust_i18n::t;

pub struct MakeSheetErrorConverter;

impl MakeSheetErrorConverter {
    pub fn to_exec_error(err: MakeSheetError) -> CmdExecuteError {
        match err {
            MakeSheetError::SheetAlreadyExists => {
                CmdExecuteError::Error(t!("make_sheet_error.sheet_already_exists").to_string())
            }
            MakeSheetError::SheetNotFound => {
                CmdExecuteError::Error(t!("make_sheet_error.sheet_not_found").to_string())
            }
            MakeSheetError::Io(error) => CmdExecuteError::Io(error),
            MakeSheetError::Other(error) => CmdExecuteError::Error(
                t!("make_sheet_error.other", error = error.to_string()).to_string(),
            ),
        }
    }
}
