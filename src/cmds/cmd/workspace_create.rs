use crate::{
    cmd_output,
    cmds::{
        arg::path::JVPathArgument, collect::empty::JVEmptyCollect,
        converter::workspace_operation_error::JVWorkspaceOperationErrorConverter,
        r#in::workspace_create::JVWorkspaceCreateInput, out::none::JVNoneOutput,
    },
    systems::{
        cmd::{
            cmd_system::JVCommandContext,
            errors::{CmdExecuteError, CmdPrepareError},
        },
        helpdoc::helpdoc_viewer,
    },
};
use cmd_system_macros::exec;
use just_enough_vcs::system::workspace::func::create_workspace;
use std::any::TypeId;

pub struct JVWorkspaceCreateCommand;
type Cmd = JVWorkspaceCreateCommand;
type Arg = JVPathArgument;
type In = JVWorkspaceCreateInput;
type Collect = JVEmptyCollect;

async fn help_str() -> String {
    helpdoc_viewer::display("commands/workspace/create").await;
    String::new()
}

async fn prepare(args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    Ok(JVWorkspaceCreateInput {
        path: args.path.clone(),
    })
}

async fn collect(_args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    Ok(JVEmptyCollect)
}

#[exec]
async fn exec(
    input: In,
    _collect: Collect,
) -> Result<(Box<dyn std::any::Any + Send + 'static>, TypeId), CmdExecuteError> {
    create_workspace(input.path)
        .await
        .map_err(JVWorkspaceOperationErrorConverter::to_exec_error)?;

    cmd_output!(JVNoneOutput => JVNoneOutput)
}

crate::command_template!();
