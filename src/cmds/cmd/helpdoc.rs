use crate::{
    cmd_output,
    cmds::{
        arg::helpdoc::JVHelpdocArgument, collect::empty::JVEmptyCollect,
        r#in::helpdoc::JVHelpdocInput, out::none::JVNoneOutput,
    },
    systems::{
        cmd::{
            cmd_system::{AnyOutput, JVCommandContext},
            errors::{CmdExecuteError, CmdPrepareError},
        },
        helpdoc::{DEFAULT_HELPDOC, helpdoc_viewer},
    },
};
use cmd_system_macros::exec;

pub struct JVHelpdocCommand;
type Cmd = JVHelpdocCommand;
type Arg = JVHelpdocArgument;
type In = JVHelpdocInput;
type Collect = JVEmptyCollect;

async fn help_str() -> String {
    helpdoc_viewer::display("commands/helpdoc").await;
    String::new()
}

async fn prepare(args: &Arg, ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    Ok(JVHelpdocInput {
        name: args.doc_name.clone().unwrap_or(DEFAULT_HELPDOC.to_string()),
        lang: ctx.lang.clone(),
    })
}

async fn collect(_args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    Ok(JVEmptyCollect)
}

#[exec]
async fn exec(input: In, _collect: Collect) -> Result<AnyOutput, CmdExecuteError> {
    helpdoc_viewer::display_with_lang(&input.name.as_str(), &input.lang.as_str()).await;
    cmd_output!(JVNoneOutput => JVNoneOutput)
}

crate::command_template!();
