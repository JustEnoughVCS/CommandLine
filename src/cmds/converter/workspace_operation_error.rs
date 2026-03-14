use crate::systems::cmd::errors::CmdExecuteError;
use just_enough_vcs::system::workspace::workspace::error::WorkspaceOperationError;

pub struct JVWorkspaceOperationErrorConverter;

impl JVWorkspaceOperationErrorConverter {
    pub fn to_exec_error(err: WorkspaceOperationError) -> CmdExecuteError {
        match err {
            WorkspaceOperationError::Io(error) => CmdExecuteError::Io(error),
            WorkspaceOperationError::Other(msg) => CmdExecuteError::Error(msg),
            WorkspaceOperationError::ConfigNotFound => {
                CmdExecuteError::Error("Config not found".to_string())
            }
            WorkspaceOperationError::WorkspaceNotFound => {
                CmdExecuteError::Error("Workspace not found".to_string())
            }
            WorkspaceOperationError::HandleLock(handle_lock_error) => {
                CmdExecuteError::Error(format!("Handle lock error: {}", handle_lock_error))
            }
            WorkspaceOperationError::DataRead(data_read_error) => {
                CmdExecuteError::Error(format!("Data read error: {}", data_read_error))
            }
            WorkspaceOperationError::DataWrite(data_write_error) => {
                CmdExecuteError::Error(format!("Data write error: {}", data_write_error))
            }
            WorkspaceOperationError::DataApply(data_apply_error) => {
                CmdExecuteError::Error(format!("Data apply error: {}", data_apply_error))
            }
        }
    }
}
