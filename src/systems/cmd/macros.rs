#[macro_export]
/// # JVCS_CLI Command Definition Macro
///
/// ## Import
///
/// Add the following macro to your code
///
/// ```ignore
/// crate::command_template!();
/// ```
///
/// Then paste the following content into your code
///
/// ```ignore
/// use crate::{
///     cmd_output,
///     cmds::{
///         arg::empty::JVEmptyArgument, collect::empty::JVEmptyCollect, r#in::empty::JVEmptyInput,
///         out::none::JVNoneOutput,
///     },
///     systems::{
///         cmd::{
///             cmd_system::{AnyOutput, JVCommandContext},
///             errors::{CmdExecuteError, CmdPrepareError},
///         },
///         helpdoc::helpdoc_viewer,
///     },
/// };
/// use cmd_system_macros::exec;
///
/// /// Define command type
/// /// Names should match the file name in the following format:
/// /// custom.rs matches JVCustomCommand, invoked using `jv custom <args...>`
/// /// get_data.rs matches JVGetDataCommand, invoked using `jv get data <args...>`
/// pub struct JVCustomCommand;
///
/// /// Command type, should match the definition above
/// type Cmd = JVCustomCommand;
///
/// /// Specify Argument
/// /// ```ignore
/// /// #[derive(Parser, Debug)]
/// /// pub struct JVCustomArgument;
/// /// ```
/// type Arg = JVEmptyArgument;
///
/// /// Specify InputData
/// /// ```ignore
/// /// pub struct JVCustomInput;
/// /// ```
/// type In = JVEmptyInput;
///
/// /// Specify CollectData
/// /// ```ignore
/// /// pub struct JVCustomCollect;
/// /// ```
/// type Collect = JVEmptyCollect;
///
/// /// Return a string, rendered when the user needs help (command specifies `--help` or syntax error)
/// async fn help_str() -> String {
///     // Write your documentation in `./resources/helpdoc`
///     // Use the format `title.lang.md`
///     helpdoc_viewer::display("commands/custom_command").await;
///     String::new()
/// }
///
/// /// Preparation phase, preprocess user input and convert to a data format friendly for the execution phase
/// async fn prepare(_args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
///     Ok(JVEmptyInput)
/// }
///
/// /// Collect necessary local information for execution
/// async fn collect(_args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
///     Ok(JVEmptyCollect)
/// }
///
/// /// Execution phase, call core layer or other custom logic
/// #[exec]
/// async fn exec(_input: In, _collect: Collect) -> Result<AnyOutput, CmdExecuteError> {
///     cmd_output!(JVNoneOutput => JVNoneOutput)
/// }
/// ```
///
/// Of course, you can also use the comment-free version
///
/// ```ignore
/// use crate::{
///     cmd_output,
///     cmds::{
///         arg::empty::JVEmptyArgument, collect::empty::JVEmptyCollect, r#in::empty::JVEmptyInput,
///         out::none::JVNoneOutput,
///     },
///     systems::{
///         cmd::{
///             cmd_system::{AnyOutput, JVCommandContext},
///             errors::{CmdExecuteError, CmdPrepareError},
///         },
///         helpdoc::helpdoc_viewer,
///     },
/// };
/// use cmd_system_macros::exec;
///
/// pub struct JVCustomCommand;
/// type Cmd = JVCustomCommand;
/// type Arg = JVEmptyArgument;
/// type In = JVEmptyInput;
/// type Collect = JVEmptyCollect;
///
/// async fn help_str() -> String {
///     helpdoc_viewer::display("commands/custom_command").await;
///     String::new()
/// }
///
/// async fn prepare(_args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
///     Ok(JVEmptyInput)
/// }
///
/// async fn collect(_args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
///     Ok(JVEmptyCollect)
/// }
///
/// #[exec]
/// async fn exec(_input: In, _collect: Collect) -> Result<AnyOutput, CmdExecuteError> {
///     cmd_output!(JVNoneOutput => JVNoneOutput)
/// }
/// ```
macro_rules! command_template {
    () => {
        impl $crate::systems::cmd::cmd_system::JVCommand<Arg, In, Collect> for Cmd {
            async fn get_help_str() -> String {
                help_str().await
            }

            async fn prepare(
                args: &Arg,
                ctx: &$crate::systems::cmd::cmd_system::JVCommandContext,
            ) -> Result<In, $crate::systems::cmd::errors::CmdPrepareError> {
                prepare(args, ctx).await
            }

            async fn collect(
                args: &Arg,
                ctx: &$crate::systems::cmd::cmd_system::JVCommandContext,
            ) -> Result<Collect, $crate::systems::cmd::errors::CmdPrepareError> {
                collect(args, ctx).await
            }

            async fn exec(
                input: In,
                collect: Collect,
            ) -> Result<
                $crate::systems::cmd::cmd_system::AnyOutput,
                $crate::systems::cmd::errors::CmdExecuteError,
            > {
                exec(input, collect).await
            }

            fn get_output_type_mapping() -> std::collections::HashMap<String, std::any::TypeId> {
                get_output_type_mapping()
            }
        }
    };
}

#[macro_export]
/// The `cmd_output!` macro should be used in the `exec` function of a command.
/// It is responsible for wrapping the execution result into a format that can be matched by the specified renderer.
///
/// Note: This macro is not only a tool for simplifying output but also a necessary component of the JvcsCLI code generation process.
/// Therefore, all command outputs must be returned through this macro.
macro_rules! cmd_output {
    ($t:ty => $v:expr) => {{
        let checked_value: $t = $v;
        Ok((
            Box::new(checked_value) as Box<dyn std::any::Any + Send + 'static>,
            std::any::TypeId::of::<$t>(),
        ))
    }};
}

#[macro_export]
/// The `early_cmd_output!` macro should be used in the `prepare` or `collect` function of a command.
/// It allows returning a valid output result early during the preparation or collection phase, instead of continuing to the `exec` phase.
/// This is suitable for situations where certain commands can determine the final output in the early stages.
///
/// Note: This macro is used for early return of successful output and should not replace error handling.
/// All command outputs must be returned through this macro or the `cmd_output!` macro.
macro_rules! early_cmd_output {
    ($t:ty => $v:expr) => {{
        let checked_value: $t = $v;
        Err($crate::systems::cmd::errors::CmdPrepareError::EarlyOutput((
            Box::new(checked_value) as Box<dyn std::any::Any + Send + 'static>,
            std::any::TypeId::of::<$t>(),
        )))
    }};
}
