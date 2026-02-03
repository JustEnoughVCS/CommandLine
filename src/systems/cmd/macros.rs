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
/// use cmd_system_macros::exec;
/// use crate::{
///     cmd_output,
///     systems::cmd::{
///         cmd_system::JVCommandContext,
///         errors::{CmdExecuteError, CmdPrepareError},
///         workspace_reader::LocalWorkspaceReader,
///     },
/// };
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
/// type Arg = JVCustomArgument;
///
/// /// Specify InputData
/// /// ```ignore
/// /// pub struct JVCustomInput;
/// /// ```
/// type In = JVCustomInput;
///
/// /// Specify CollectData
/// /// ```ignore
/// /// pub struct JVCustomCollect;
/// /// ```
/// type Collect = JVCustomCollect;
///
/// /// Return a string, rendered when the user needs help (command specifies `--help` or syntax error)
/// fn help_str() -> String {
///     todo!()
/// }
///
/// /// Preparation phase, preprocess user input and convert to a data format friendly for the execution phase
/// async fn prepare(args: &Arg, ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
///     todo!()
/// }
///
/// /// Collect necessary local information for execution
/// async fn collect(args: &Arg, ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
///     let reader = LocalWorkspaceReader::default();
///     todo!()
/// }
///
/// /// Execution phase, call core layer or other custom logic
/// #[exec]
/// async fn exec(
///     input: In,
///     collect: Collect,
/// ) -> Result<(Box<dyn std::any::Any + Send + 'static>, String), CmdExecuteError> {
///     todo!();
///
///     // Use the following method to return results
///     cmd_output!(output, JVCustomOutput)
/// }
/// ```
///
/// Of course, you can also use the comment-free version
///
/// ```ignore
/// use cmd_system_macros::exec;
/// use crate::{
///     cmd_output,
///     systems::cmd::{
///         cmd_system::JVCommandContext,
///         errors::{CmdExecuteError, CmdPrepareError},
///         workspace_reader::LocalWorkspaceReader,
///     },
/// };
///
/// pub struct JVCustomCommand;
/// type Cmd = JVCustomCommand;
/// type Arg = JVCustomArgument;
/// type In = JVCustomInput;
/// type Collect = JVCustomCollect;
///
/// fn help_str() -> String {
///     todo!()
/// }
///
/// async fn prepare(args: &Arg, ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
///     todo!()
/// }
///
/// async fn collect(args: &Arg, ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
///     let reader = LocalWorkspaceReader::default();
///     todo!()
/// }
///
/// #[exec]
/// async fn exec(
///     input: In,
///     collect: Collect,
/// ) -> Result<(Box<dyn std::any::Any + Send + 'static>, String), CmdExecuteError> {
///     todo!();
///     cmd_output!(output, JVCustomOutput)
/// }
/// ```
macro_rules! command_template {
    () => {
        impl $crate::systems::cmd::cmd_system::JVCommand<Arg, In, Collect> for Cmd {
            fn get_help_str() -> String {
                help_str()
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
                (Box<dyn std::any::Any + Send + 'static>, String),
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
macro_rules! cmd_output {
    ($v:expr, $t:ty) => {
        Ok((Box::new($v), stringify!($t).to_string()))
    };
}
