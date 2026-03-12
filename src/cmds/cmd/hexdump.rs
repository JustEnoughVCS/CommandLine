use std::any::TypeId;

use crate::{
    cmd_output,
    cmds::{
        arg::single_file::JVSingleFileArgument, collect::single_file::JVSingleFileCollect,
        r#in::empty::JVEmptyInput, out::hex::JVHexOutput,
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
use tokio::fs;

pub struct JVHexdumpCommand;
type Cmd = JVHexdumpCommand;
type Arg = JVSingleFileArgument;
type In = JVEmptyInput;
type Collect = JVSingleFileCollect;

async fn help_str() -> String {
    helpdoc_viewer::display("commands/hexdump").await;
    String::new()
}

async fn prepare(_args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    Ok(In {})
}

async fn collect(args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    let file = &args.file;
    let data = fs::read(file).await?;
    Ok(Collect { data })
}

#[exec]
async fn exec(
    _input: In,
    collect: Collect,
) -> Result<(Box<dyn std::any::Any + Send + 'static>, TypeId), CmdExecuteError> {
    let output = JVHexOutput { data: collect.data };
    cmd_output!(JVHexOutput => output)
}

crate::command_template!();
