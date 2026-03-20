use crate::{
    cmd_output,
    cmds::{
        arg::workspace_alias::JVWorkspaceAliasArgument,
        collect::workspace::JVWorkspaceCollect,
        converter::workspace_operation_error::JVWorkspaceOperationErrorConverter,
        r#in::workspace_alias::{JVWorkspaceAliasInput, JVWorkspaceAliasInputMode},
        out::{alias_query::JVAliasQueryOutput, none::JVNoneOutput},
    },
    systems::{
        cmd::{
            cmd_system::{AnyOutput, JVCommandContext},
            errors::{CmdExecuteError, CmdPrepareError},
        },
        helpdoc::helpdoc_viewer,
    },
};
use cmd_system_macros::exec;
use just_enough_vcs::system::workspace::workspace::manager::WorkspaceManager;
use log::trace;
use rust_i18n::t;

pub struct JVWorkspaceAliasCommand;
type Cmd = JVWorkspaceAliasCommand;
type Arg = JVWorkspaceAliasArgument;
type In = JVWorkspaceAliasInput;
type Collect = JVWorkspaceCollect;

async fn help_str() -> String {
    helpdoc_viewer::display("commands/workspace/alias").await;
    String::new()
}

async fn prepare(args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    let mode = match (args.insert, args.erase, args.to) {
        (false, true, _) => JVWorkspaceAliasInputMode::Erase(args.id),
        (true, false, Some(to)) => JVWorkspaceAliasInputMode::Insert(args.id, to),
        (false, false, _) => JVWorkspaceAliasInputMode::None(args.id),
        _ => {
            return Err(CmdPrepareError::Error(
                t!("workspace_alias.error.undeclared_input").to_string(),
            ));
        }
    };

    Ok(JVWorkspaceAliasInput {
        query: args.query,
        mode,
    })
}

async fn collect(_args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    Ok(JVWorkspaceCollect {
        manager: WorkspaceManager::new(),
    })
}

#[exec]
async fn exec(input: In, collect: Collect) -> Result<AnyOutput, CmdExecuteError> {
    let manager = collect.manager;
    let index_source = match input.mode {
        JVWorkspaceAliasInputMode::Insert(index, to) => {
            trace!("Inserting alias for index: {}, to: {}", index, to);
            manager
                .write_id_alias(index, to)
                .await
                .map_err(JVWorkspaceOperationErrorConverter::to_exec_error)?;
            index
        }
        JVWorkspaceAliasInputMode::Erase(index) => {
            trace!("Erasing alias for index: {}", index);
            manager
                .delete_id_alias(index)
                .await
                .map_err(JVWorkspaceOperationErrorConverter::to_exec_error)?;
            index
        }
        JVWorkspaceAliasInputMode::None(index) => {
            trace!("No alias operation requested");
            // Do nothing
            index
        }
    };

    match (input.query, index_source) {
        (true, index) => {
            trace!("Querying alias for index: {}", index);
            let remote = manager
                .try_convert_to_remote(index)
                .await
                .map_err(JVWorkspaceOperationErrorConverter::to_exec_error)?;
            trace!(
                "Alias query result - local: {}, remote: {:?}",
                index, remote
            );
            cmd_output!(JVAliasQueryOutput => JVAliasQueryOutput { local: index, remote })
        }
        _ => {
            trace!("No alias query requested or no index source available");
            cmd_output!(JVNoneOutput => JVNoneOutput)
        }
    }
}

crate::command_template!();
