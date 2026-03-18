use crate::systems::cmd::errors::CmdExecuteError;
use just_enough_vcs::system::workspace::workspace::error::WorkspaceOperationError;
use rust_i18n::t;

pub struct JVWorkspaceOperationErrorConverter;

impl JVWorkspaceOperationErrorConverter {
    pub fn to_exec_error(err: WorkspaceOperationError) -> CmdExecuteError {
        match err {
            WorkspaceOperationError::Io(error) => CmdExecuteError::Io(error),
            WorkspaceOperationError::Other(msg) => CmdExecuteError::Error(msg),
            WorkspaceOperationError::ConfigNotFound => {
                CmdExecuteError::Error(t!("workspace_operation_error.config_not_found").to_string())
            }
            WorkspaceOperationError::WorkspaceNotFound => CmdExecuteError::Error(
                t!("workspace_operation_error.workspace_not_found").to_string(),
            ),
            WorkspaceOperationError::HandleLock(handle_lock_error) => CmdExecuteError::Error(
                t!(
                    "workspace_operation_error.handle_lock",
                    error = handle_lock_error
                )
                .to_string(),
            ),
            WorkspaceOperationError::DataRead(data_read_error) => CmdExecuteError::Error(
                t!(
                    "workspace_operation_error.data_read",
                    error = data_read_error
                )
                .to_string(),
            ),
            WorkspaceOperationError::DataWrite(data_write_error) => CmdExecuteError::Error(
                t!(
                    "workspace_operation_error.data_write",
                    error = data_write_error
                )
                .to_string(),
            ),
            WorkspaceOperationError::DataApply(data_apply_error) => CmdExecuteError::Error(
                t!(
                    "workspace_operation_error.data_apply",
                    error = data_apply_error
                )
                .to_string(),
            ),
            WorkspaceOperationError::IDAliasError(id_alias_error) => CmdExecuteError::Error(
                t!(
                    "workspace_operation_error.id_alias_error",
                    error = id_alias_error
                )
                .to_string(),
            ),
        }
    }
}
