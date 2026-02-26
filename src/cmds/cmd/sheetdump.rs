use std::any::TypeId;

use crate::{
    cmd_output,
    cmds::{
        arg::sheetdump::JVSheetdumpArgument,
        collect::sheetdump::JVSheetdumpCollect,
        r#in::sheetdump::JVSheetdumpInput,
        out::{mappings::JVMappingsOutput, mappings_pretty::JVMappingsPrettyOutput},
    },
    systems::cmd::{
        cmd_system::JVCommandContext,
        errors::{CmdExecuteError, CmdPrepareError},
    },
};
use cmd_system_macros::exec;
use just_enough_vcs::system::sheet_system::{
    mapping::LocalMapping,
    sheet::{SheetData, error::ReadSheetDataError},
};

pub struct JVSheetdumpCommand;
type Cmd = JVSheetdumpCommand;
type Arg = JVSheetdumpArgument;
type In = JVSheetdumpInput;
type Collect = JVSheetdumpCollect;

fn help_str() -> String {
    todo!()
}

async fn prepare(args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    Ok(In {
        sort: !args.no_sort,
        pretty: !args.no_pretty,
    })
}

async fn collect(args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    let mut sheet = SheetData::empty();

    sheet
        .full_read(&args.sheet_file)
        .await
        .map_err(|e| match e {
            ReadSheetDataError::IOErr(error) => CmdPrepareError::Io(error),
        })?;

    Ok(Collect { sheet: sheet })
}

#[exec]
async fn exec(
    input: In,
    collect: Collect,
) -> Result<(Box<dyn std::any::Any + Send + 'static>, TypeId), CmdExecuteError> {
    let mappings = collect.sheet.mappings();
    let mut mappings_vec = mappings.iter().cloned().collect::<Vec<LocalMapping>>();
    if input.sort {
        mappings_vec.sort();
    }

    if input.pretty {
        cmd_output!(JVMappingsPrettyOutput => JVMappingsPrettyOutput {
            mappings: mappings_vec,
        })
    } else {
        cmd_output!(JVMappingsOutput => JVMappingsOutput {
            mappings: mappings_vec,
        })
    }
}

crate::command_template!();
