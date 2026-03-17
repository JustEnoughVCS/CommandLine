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
                crate::systems::cmd::cmd_system::AnyOutput,
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
/// `cmd_output!` 宏应在命令的 `exec` 函数中使用。
/// 它负责将执行结果包装成能被指定渲染器匹配的格式。
///
/// 注意：该宏不仅是简化输出的工具，更是 JvcsCLI 代码生成流程的必要组成部分。
/// 因此，所有命令的输出都必须通过此宏返回。
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
/// `early_cmd_output!` 宏应在命令的 `prepare` 或 `collect` 函数中使用。
/// 它负责将执行结果包装成能返回到核心
///
/// 注意：该宏不仅是简化输出的工具，更是 JvcsCLI 代码生成流程的必要组成部分。
/// 因此，所有命令的输出都必须通过此宏返回。
macro_rules! early_cmd_output {
    ($t:ty => $v:expr) => {{
        let checked_value: $t = $v;
        Err(crate::systems::cmd::errors::CmdPrepareError::EarlyOutput((
            Box::new(checked_value) as Box<dyn std::any::Any + Send + 'static>,
            std::any::TypeId::of::<$t>(),
        )))
    }};
}
