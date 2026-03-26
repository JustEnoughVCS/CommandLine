use crate::systems::cmd::errors::CmdExecuteError;
use just_enough_vcs::system::space::error::SpaceError;
use rust_i18n::t;

pub struct JVSpaceErrorConverter;

impl JVSpaceErrorConverter {
    pub fn to_exec_error(err: SpaceError) -> CmdExecuteError {
        match err {
            SpaceError::SpaceNotFound => {
                CmdExecuteError::Error(t!("space_error.space_not_found").trim().to_string())
            }
            SpaceError::PathFormatError(path_format_error) => CmdExecuteError::Error(
                t!("space_error.path_fmt_error", error = path_format_error)
                    .trim()
                    .to_string(),
            ),
            SpaceError::Io(error) => CmdExecuteError::Io(error),
            SpaceError::Other(msg) => CmdExecuteError::Error(msg),
        }
    }
}
