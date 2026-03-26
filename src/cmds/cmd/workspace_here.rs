use crate::{
    cmd_output,
    cmds::{
        arg::empty::JVEmptyArgument, collect::workspace::JVWorkspaceCollect,
        converter::space_error::JVSpaceErrorConverter, r#in::empty::JVEmptyInput,
        out::path::JVPathOutput,
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

pub struct JVWorkspaceHereCommand;
type Cmd = JVWorkspaceHereCommand;
type Arg = JVEmptyArgument;
type In = JVEmptyInput;
type Collect = JVWorkspaceCollect;

async fn help_str() -> String {
    helpdoc_viewer::display("commands/workspace_here").await;
    String::new()
}

async fn prepare(_args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    Ok(JVEmptyInput)
}

async fn collect(_args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    Ok(JVWorkspaceCollect {
        manager: WorkspaceManager::new(),
    })
}

#[exec]
async fn exec(_input: In, collect: Collect) -> Result<AnyOutput, CmdExecuteError> {
    let path = collect
        .manager
        .get_space()
        .space_dir_current()
        .map_err(JVSpaceErrorConverter::to_exec_error)?;
    cmd_output!(JVPathOutput => JVPathOutput { path })
}

crate::command_template!();
