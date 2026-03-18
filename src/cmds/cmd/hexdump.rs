use crate::{
    cmd_output,
    cmds::{
        arg::single_file::JVSingleFileArgument,
        collect::single_file::JVSingleFileCollect,
        r#in::empty::JVEmptyInput,
        out::{hex::JVHexOutput, none::JVNoneOutput},
    },
    early_cmd_output,
    systems::{
        cmd::{
            cmd_system::{AnyOutput, JVCommandContext},
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

async fn collect(args: &Arg, ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    let data = if let Some(ref stdin_path) = ctx.stdin_path {
        fs::read(stdin_path).await?
    } else if let Some(ref stdin_data) = ctx.stdin_data {
        stdin_data.clone()
    } else if let Some(path) = &args.file {
        fs::read(&path).await?
    } else {
        // No path input, exit early
        return early_cmd_output!(JVNoneOutput => JVNoneOutput);
    };
    Ok(Collect { data })
}

#[exec]
async fn exec(_input: In, collect: Collect) -> Result<AnyOutput, CmdExecuteError> {
    let output = JVHexOutput { data: collect.data };
    cmd_output!(JVHexOutput => output)
}

crate::command_template!();
