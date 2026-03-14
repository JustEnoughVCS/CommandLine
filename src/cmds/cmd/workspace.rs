use crate::{
    cmd_output,
    cmds::{
        arg::empty::JVEmptyArgument, collect::empty::JVEmptyCollect, r#in::empty::JVEmptyInput,
        out::none::JVNoneOutput,
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
use std::any::TypeId;

pub struct JVWorkspaceCommand;
type Cmd = JVWorkspaceCommand;
type Arg = JVEmptyArgument;
type In = JVEmptyInput;
type Collect = JVEmptyCollect;

async fn help_str() -> String {
    helpdoc_viewer::display("commands/workspace").await;
    String::new()
}

async fn prepare(_args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    Ok(JVEmptyInput)
}

async fn collect(_args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    Ok(JVEmptyCollect)
}

#[exec]
async fn exec(
    _input: In,
    _collect: Collect,
) -> Result<(Box<dyn std::any::Any + Send + 'static>, TypeId), CmdExecuteError> {
    cmd_output!(JVNoneOutput => JVNoneOutput)
}

crate::command_template!();
