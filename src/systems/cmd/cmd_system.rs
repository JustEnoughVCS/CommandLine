use serde::Serialize;

use crate::{
    r_println,
    systems::cmd::{
        errors::{CmdExecuteError, CmdPrepareError, CmdProcessError, CmdRenderError},
        renderer::{JVRenderResult, JVResultRenderer},
    },
};
use std::future::Future;

pub struct JVCommandContext {
    pub help: bool,
    pub confirmed: bool,
}

pub trait JVCommand<Argument, Input, Collect, Output, Renderer>
where
    Argument: clap::Parser + Send,
    Input: Send,
    Output: Serialize + Send + Sync,
    Collect: Send,
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
            include!("_renderers.rs")
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
    fn process_with_renderer<R: JVResultRenderer<Output> + Send>(
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

            let (input, collect) = match tokio::try_join!(
                Self::prepare(&parsed_args, &ctx),
                Self::collect(&parsed_args, &ctx)
            ) {
                Ok((input, collect)) => (input, collect),
                Err(e) => return Err(CmdProcessError::from(e)),
            };

            let output = match Self::exec(input, collect).await {
                Ok(output) => output,
                Err(e) => return Err(CmdProcessError::from(e)),
            };

            match R::render(&output).await {
                Ok(r) => Ok(r),
                Err(e) => Err(CmdProcessError::from(e)),
            }
        }
    }
    /// Prepare
    /// Converts Argument input into parameters readable during the execution phase
    fn prepare(
        args: &Argument,
        ctx: &JVCommandContext,
    ) -> impl Future<Output = Result<Input, CmdPrepareError>> + Send;

    /// Resource collection
    /// Reads required resources and sends them to the `exec` function
    fn collect(
        args: &Argument,
        ctx: &JVCommandContext,
    ) -> impl Future<Output = Result<Collect, CmdPrepareError>> + Send;

    /// Execute
    /// Executes the results obtained from `prepare` and `collect`
    /// Returns data that can be used for rendering
    fn exec(
        input: Input,
        collect: Collect,
    ) -> impl Future<Output = Result<Output, CmdExecuteError>> + Send;
}
