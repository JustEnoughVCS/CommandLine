use clap::Parser;
use serde::Serialize;

use crate::cmd::{
    cmd_system::{JVCommand, JVCommandContext},
    errors::{CmdExecuteError, CmdPrepareError, CmdRenderError},
    renderer::{JVRenderResult, JVResultRenderer},
};

pub struct JVStatusCommand;

#[derive(Parser, Debug)]
pub struct JVStatusArgument;

pub struct JVStatusInput;

#[derive(Serialize)]
pub struct JVStatusOutput;

impl JVCommand<JVStatusArgument, JVStatusInput, JVStatusOutput, JVStatusRenderer>
    for JVStatusCommand
{
    async fn prepare(
        _args: JVStatusArgument,
        _ctx: JVCommandContext,
    ) -> Result<JVStatusInput, CmdPrepareError> {
        Ok(JVStatusInput)
    }

    async fn exec(args: JVStatusInput) -> Result<JVStatusOutput, CmdExecuteError> {
        todo!()
    }

    fn get_help_str() -> String {
        "".to_string()
    }
}

pub struct JVStatusRenderer;

impl JVResultRenderer<JVStatusOutput> for JVStatusRenderer {
    async fn render(data: &JVStatusOutput) -> Result<JVRenderResult, CmdRenderError> {
        todo!()
    }
}
