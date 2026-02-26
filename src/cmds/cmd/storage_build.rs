use crate::{
    cmd_output,
    cmds::{
        arg::storage_build::JVStorageBuildArgument, collect::empty::JVEmptyCollect,
        r#in::storage_rw::JVStorageRWInput, out::none::JVNoneOutput,
    },
    systems::cmd::{
        cmd_system::JVCommandContext,
        errors::{CmdExecuteError, CmdPrepareError},
    },
};
use cli_utils::display::md;
use cmd_system_macros::exec;
use just_enough_vcs::system::storage_system::{error::StorageIOError, store::build_file};
use rust_i18n::t;
use std::any::TypeId;

pub struct JVStorageBuildCommand;
type Cmd = JVStorageBuildCommand;
type Arg = JVStorageBuildArgument;
type In = JVStorageRWInput;
type Collect = JVEmptyCollect;

fn help_str() -> String {
    todo!()
}

async fn prepare(args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    let output_file = match &args.output_file {
        Some(v) => v.clone(),
        None => args.index_file.clone().with_extension("unknown"),
    };

    let (input, storage, output) = just_enough_vcs::system::storage_system::store::precheck(
        args.index_file.clone(),
        args.storage.clone(),
        output_file,
    )
    .await?;

    Ok(JVStorageRWInput {
        input,
        storage,
        output,
        chunking_policy: None,
    })
}

async fn collect(_args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    Ok(Collect {})
}

#[exec]
async fn exec(
    input: In,
    _collect: Collect,
) -> Result<(Box<dyn std::any::Any + Send + 'static>, TypeId), CmdExecuteError> {
    build_file(input.input, input.storage, input.output)
        .await
        .map_err(|e| match e {
            StorageIOError::IOErr(error) => CmdExecuteError::Io(error),
            StorageIOError::HashTooShort => {
                CmdExecuteError::Error(md(t!("storage_write.hash_too_short")).to_string())
            }
        })?;

    cmd_output!(JVNoneOutput => JVNoneOutput {})
}

crate::command_template!();
