use crate::{
    cmd_output,
    cmds::{
        arg::version::JVVersionArgument, collect::compile_info::JVCompileInfoCollect,
        converter::version_in_out::JVVersionInputOutputConverter, r#in::version::JVVersionInput,
        out::version::JVVersionOutput,
    },
    data::compile_info::CompileInfo,
    systems::{
        cmd::{
            cmd_system::{AnyOutput, JVCommandContext},
            errors::{CmdExecuteError, CmdPrepareError},
        },
        helpdoc::helpdoc_viewer,
    },
};
use cmd_system_macros::exec;
use just_enough_vcs::data::compile_info::CoreCompileInfo;

pub struct JVVersionCommand;
type Cmd = JVVersionCommand;
type Arg = JVVersionArgument;
type In = JVVersionInput;
type Collect = JVCompileInfoCollect;

async fn help_str() -> String {
    helpdoc_viewer::display("commands/version").await;
    String::new()
}

async fn prepare(args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    Ok(JVVersionInput {
        show_compile_info: args.with_compile_info,
        show_banner: !args.no_banner,
    })
}

async fn collect(_args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    Ok(JVCompileInfoCollect {
        compile_info: CompileInfo::default(),
        compile_info_core: CoreCompileInfo::default(),
    })
}

#[exec]
async fn exec(input: In, collect: Collect) -> Result<AnyOutput, CmdExecuteError> {
    let output = JVVersionInputOutputConverter::merge_to_output(input, collect);
    cmd_output!(JVVersionOutput => output)
}

crate::command_template!();
