use crate::{
    cmd_output,
    cmds::{
        arg::empty::JVEmptyArgument, collect::empty::JVEmptyCollect, r#in::empty::JVEmptyInput,
        out::helpdocs::JVHelpdocsOutput,
    },
    systems::{
        cmd::{
            cmd_system::JVCommandContext,
            errors::{CmdExecuteError, CmdPrepareError},
        },
        helpdoc::{DEFAULT_HELPDOC, get_helpdoc_list, helpdoc_viewer},
    },
};
use cmd_system_macros::exec;
use std::any::TypeId;

pub struct JVHelpdocListCommand;
type Cmd = JVHelpdocListCommand;
type Arg = JVEmptyArgument;
type In = JVEmptyInput;
type Collect = JVEmptyCollect;

async fn help_str() -> String {
    helpdoc_viewer::display(DEFAULT_HELPDOC).await;
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
    let output = JVHelpdocsOutput {
        names: get_helpdoc_list().into_iter().map(String::from).collect(),
    };
    cmd_output!(JVHelpdocsOutput => output)
}

crate::command_template!();
