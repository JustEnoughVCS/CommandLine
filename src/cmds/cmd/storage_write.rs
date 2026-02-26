use crate::{
    cmd_output,
    cmds::{
        arg::storage_write::JVStorageWriteArgument, collect::empty::JVEmptyCollect,
        r#in::storage_rw::JVStorageRWInput, out::none::JVNoneOutput,
    },
    systems::cmd::{
        cmd_system::JVCommandContext,
        errors::{CmdExecuteError, CmdPrepareError},
    },
};
use cli_utils::display::md;
use cmd_system_macros::exec;
use just_enough_vcs::system::{
    constants::vault::values::vault_value_index_file_suffix,
    storage_system::{
        error::StorageIOError,
        store::{ChunkingPolicy, StorageConfig, write_file},
    },
};
use rust_i18n::t;
use std::any::TypeId;

pub struct JVStorageWriteCommand;
type Cmd = JVStorageWriteCommand;
type Arg = JVStorageWriteArgument;
type In = JVStorageRWInput;
type Collect = JVEmptyCollect;

fn help_str() -> String {
    todo!()
}

async fn prepare(args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    let output_path = match &args.output_index {
        Some(v) => v.clone(),
        None => args
            .file
            .clone()
            .with_extension(vault_value_index_file_suffix()),
    };

    // Default to using kb as the unit
    let scale = if args.gb {
        1024 * 1024 * 1024
    } else if args.mb {
        1024 * 1024
    } else if args.b {
        1
    } else {
        1024
    };

    let (input, storage, output) = just_enough_vcs::system::storage_system::store::precheck(
        args.file.clone(),
        args.storage.clone(),
        output_path,
    )
    .await?;

    let chunking_policy: ChunkingPolicy = if args.cdc_chunking > 0 {
        ChunkingPolicy::Cdc(args.cdc_chunking * scale)
    } else if args.fixed_chunking > 0 {
        ChunkingPolicy::FixedSize(args.fixed_chunking * scale)
    } else if args.line_chunking {
        ChunkingPolicy::Line
    } else {
        return Err(CmdPrepareError::Error(md(t!(
            "storage_write.unknown_chunking_policy"
        ))));
    };

    Ok(JVStorageRWInput {
        input,
        storage,
        output,
        chunking_policy: Some(chunking_policy),
    })
}

async fn collect(_args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    Ok(JVEmptyCollect {})
}

#[exec]
async fn exec(
    input: In,
    _collect: Collect,
) -> Result<(Box<dyn std::any::Any + Send + 'static>, TypeId), CmdExecuteError> {
    // There is no chance to return None in the Prepare phase, so unwrap is safe here
    let chunking_policy = input.chunking_policy.unwrap();

    write_file(
        input.input,
        input.storage,
        input.output,
        &StorageConfig { chunking_policy },
    )
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
