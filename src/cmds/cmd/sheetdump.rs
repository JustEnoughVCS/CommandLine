use crate::{
    cmd_output,
    cmds::{
        arg::sheetdump::JVSheetdumpArgument,
        collect::sheetdump::JVSheetdumpCollect,
        r#in::sheetdump::JVSheetdumpInput,
        out::{
            mappings::JVMappingsOutput, mappings_pretty::JVMappingsPrettyOutput, none::JVNoneOutput,
        },
    },
    early_cmd_output,
    systems::{
        cmd::{
            cmd_system::{AnyOutput, JVCommandContext},
            errors::{CmdExecuteError, CmdPrepareError},
        },
        helpdoc::helpdoc_viewer,
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

async fn help_str() -> String {
    helpdoc_viewer::display("commands/sheetdump").await;
    String::new()
}

async fn prepare(args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    Ok(In {
        sort: !args.no_sort,
        pretty: !args.no_pretty,
    })
}

async fn collect(args: &Arg, ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    let mut sheet = SheetData::empty();

    let path = match (&args.sheet_file, &ctx.stdin_path) {
        (Some(_), Some(stdin)) => stdin,
        (Some(file), None) => file,
        (None, Some(stdin)) => stdin,
        (None, None) => {
            return early_cmd_output!(JVNoneOutput => JVNoneOutput);
        }
    };

    sheet.full_read(path).await.map_err(|e| match e {
        ReadSheetDataError::IOErr(error) => CmdPrepareError::Io(error),
    })?;

    Ok(Collect { sheet })
}

#[exec]
async fn exec(input: In, collect: Collect) -> Result<AnyOutput, CmdExecuteError> {
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
