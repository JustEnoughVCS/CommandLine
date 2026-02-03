use crate::{
    r_println,
    systems::{
        cmd::errors::{CmdExecuteError, CmdPrepareError, CmdProcessError, CmdRenderError},
        render::{render_system::render, renderer::JVRenderResult},
    },
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    future::Future,
};

pub struct JVCommandContext {
    pub help: bool,
    pub confirmed: bool,
}

pub trait JVCommand<Argument, Input, Collect>
where
    Argument: clap::Parser + Send,
    Input: Send,
    Collect: Send,
{
    /// Get help string for the command
    fn get_help_str() -> String;

    /// Run the command and convert the result into type-agnostic serialized information,
    /// then hand it over to the universal renderer for rendering.
    /// Universal renderer: uses the renderer specified by the `--renderer` flag.
    fn process_to_renderer_override(
        args: Vec<String>,
        ctx: JVCommandContext,
        renderer_override: String,
    ) -> impl Future<Output = Result<JVRenderResult, CmdProcessError>> + Send {
        async move {
            // If the `--help` flag is used,
            // skip execution and return an error,
            // unlike `process_to_render_system`,
            // when the `--renderer` flag specifies a renderer, `--help` output is not allowed
            if ctx.help {
                return Err(CmdProcessError::RendererOverrideButRequestHelp);
            }

            let (data, type_name) = Self::process(args, ctx).await?;

            let renderer_override = renderer_override.as_str();

            // Serialize the data based on its concrete type
            let render_result = include!("../render/_override_renderer_entry.rs");

            match render_result {
                Ok(r) => Ok(r),
                Err(e) => Err(CmdProcessError::Render(e)),
            }
        }
    }

    /// Run the command and hand it over to the rendering system
    /// to select the appropriate renderer for the result
    fn process_to_render_system(
        args: Vec<String>,
        ctx: JVCommandContext,
    ) -> impl Future<Output = Result<JVRenderResult, CmdProcessError>> + Send {
        async {
            // If the `--help` flag is used,
            // skip execution and directly render help information
            if ctx.help {
                let mut r = JVRenderResult::default();
                r_println!(r, "{}", Self::get_help_str());
                return Ok(r);
            }

            let (data, id_str) = Self::process(args, ctx).await?;
            match render(data, id_str).await {
                Ok(r) => Ok(r),
                Err(e) => Err(CmdProcessError::Render(e)),
            }
        }
    }

    fn process(
        args: Vec<String>,
        ctx: JVCommandContext,
    ) -> impl Future<Output = Result<(Box<dyn Any + Send + 'static>, String), CmdProcessError>> + Send
    {
        async move {
            let mut full_args = vec!["jv".to_string()];

            full_args.extend(args);

            let parsed_args = match Argument::try_parse_from(full_args) {
                Ok(args) => args,
                Err(_) => return Err(CmdProcessError::ParseError(Self::get_help_str())),
            };

            let (input, collect) = match tokio::try_join!(
                Self::prepare(&parsed_args, &ctx),
                Self::collect(&parsed_args, &ctx)
            ) {
                Ok((input, collect)) => (input, collect),
                Err(e) => return Err(CmdProcessError::from(e)),
            };

            let data = match Self::exec(input, collect).await {
                Ok(output) => output,
                Err(e) => return Err(CmdProcessError::from(e)),
            };

            Ok(data)
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
    ) -> impl Future<Output = Result<(Box<dyn Any + Send + 'static>, String), CmdExecuteError>> + Send;

    /// Get output type mapping
    fn get_output_type_mapping() -> HashMap<String, TypeId>;
}
