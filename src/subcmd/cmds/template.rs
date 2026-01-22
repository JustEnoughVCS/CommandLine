use clap::Parser;
use serde::Serialize;

use crate::subcmd::{
    cmd::JVCommand,
    errors::{CmdExecuteError, CmdPrepareError, CmdRenderError},
    renderer::{JVRenderResult, JVResultRenderer},
};

pub struct JVUnknownCommand;

#[derive(Parser, Debug)]
pub struct JVUnknownArgument;

pub struct JVUnknownInput;

#[derive(Serialize)]
pub struct JVUnknownOutput;

impl JVCommand<JVUnknownArgument, JVUnknownInput, JVUnknownOutput, JVStatusRenderer>
    for JVUnknownCommand
{
    async fn prepare(
        _args: JVUnknownArgument,
        _ctx: JVCommandContext,
    ) -> Result<JVUnknownInput, CmdPrepareError> {
        todo!()
    }

    async fn exec(_args: JVUnknownInput) -> Result<JVUnknownOutput, CmdExecuteError> {
        todo!()
    }

    fn get_help_str() -> String {
        "".to_string()
    }
}

pub struct JVStatusRenderer;

impl JVResultRenderer<JVUnknownOutput> for JVStatusRenderer {
    async fn render(_data: &JVUnknownOutput) -> Result<JVRenderResult, CmdRenderError> {
        todo!()
    }
}
