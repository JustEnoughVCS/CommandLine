use serde::Serialize;

use crate::{
    cmd::{
        errors::{CmdExecuteError, CmdPrepareError, CmdProcessError, CmdRenderError},
        renderer::{JVRenderResult, JVResultRenderer},
    },
    r_println,
};
use std::future::Future;

pub struct JVCommandContext {
    pub help: bool,
    pub confirmed: bool,
}

pub trait JVCommand<Argument, Input, Output, Renderer>
where
    Argument: clap::Parser + Send + Sync,
    Input: Send + Sync,
    Output: Serialize + Send + Sync,
    Renderer: JVResultRenderer<Output> + Send + Sync,
{
    /// Get help string for the command
    fn get_help_str() -> String;

    /// Process the command with a specified renderer, performing any necessary post-execution processing
    fn process_with_renderer_flag(
        args: Vec<String>,
        ctx: JVCommandContext,
        renderer: String,
    ) -> impl Future<Output = Result<JVRenderResult, CmdProcessError>> + Send
    where
        Self: Sync,
    {
        async move {
            let renderer_str = renderer.as_str();
            include!("renderers/_renderers.rs")
        }
    }

    /// performing any necessary post-execution processing
    fn process(
        args: Vec<String>,
        ctx: JVCommandContext,
    ) -> impl Future<Output = Result<JVRenderResult, CmdProcessError>> + Send
    where
        Self: Sync,
    {
        Self::process_with_renderer::<Renderer>(args, ctx)
    }

    /// Process the command output with a custom renderer,
    /// performing any necessary post-execution processing
    fn process_with_renderer<R: JVResultRenderer<Output> + Send + Sync>(
        args: Vec<String>,
        ctx: JVCommandContext,
    ) -> impl Future<Output = Result<JVRenderResult, CmdProcessError>> + Send
    where
        Self: Sync,
    {
        async move {
            let mut full_args = vec!["jv".to_string()];
            full_args.extend(args);
            let parsed_args = match Argument::try_parse_from(full_args) {
                Ok(args) => args,
                Err(_) => return Err(CmdProcessError::ParseError(Self::get_help_str())),
            };
            // If the help flag is used, skip execution and directly print help
            if ctx.help {
                let mut r = JVRenderResult::default();
                r_println!(r, "{}", Self::get_help_str());
                return Ok(r);
            }
            let input = match Self::prepare(parsed_args, ctx).await {
                Ok(input) => input,
                Err(e) => return Err(CmdProcessError::from(e)),
            };
            let output = match Self::exec(input).await {
                Ok(output) => output,
                Err(e) => return Err(CmdProcessError::from(e)),
            };
            match R::render(&output).await {
                Ok(r) => Ok(r),
                Err(e) => Err(CmdProcessError::from(e)),
            }
        }
    }

    /// Prepare to run the command,
    /// converting Clap input into the command's supported input
    fn prepare(
        args: Argument,
        ctx: JVCommandContext,
    ) -> impl Future<Output = Result<Input, CmdPrepareError>> + Send;

    /// Run the command phase,
    /// returning an output structure, waiting for rendering
    fn exec(input: Input) -> impl Future<Output = Result<Output, CmdExecuteError>> + Send;
}
