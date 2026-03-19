use crate::{
    cmd_output,
    cmds::{
        arg::workspace_sheet::JVWorkspaceSheetArgument,
        collect::workspace::JVWorkspaceCollect,
        converter::make_sheet_error::MakeSheetErrorConverter,
        r#in::workspace_sheet::JVWorkspaceSheetInput,
        out::{none::JVNoneOutput, path::JVPathOutput, string_vcs::JVStringVecOutput},
    },
    systems::{
        cmd::{
            cmd_system::{AnyOutput, JVCommandContext},
            errors::{CmdExecuteError, CmdPrepareError},
        },
        helpdoc::helpdoc_viewer,
    },
};
use cmd_system_macros::exec;
use just_enough_vcs::system::workspace::workspace::manager::WorkspaceManager;
use rust_i18n::t;

pub struct JVWorkspaceSheetCommand;
type Cmd = JVWorkspaceSheetCommand;
type Arg = JVWorkspaceSheetArgument;
type In = JVWorkspaceSheetInput;
type Collect = JVWorkspaceCollect;

async fn help_str() -> String {
    helpdoc_viewer::display("commands/workspace/sheet").await;
    String::new()
}

async fn prepare(_args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    let input = match (_args.new, _args.delete, _args.list_all, _args.print_path) {
        (true, false, false, false) => {
            let name = _args.name.as_ref().ok_or_else(|| {
                CmdPrepareError::Error(
                    t!("workspace_sheet.error.sheet_name_required_for_new")
                        .trim()
                        .to_string(),
                )
            })?;
            JVWorkspaceSheetInput::Add(name.clone())
        }
        (false, true, false, false) => {
            let name = _args.name.as_ref().ok_or_else(|| {
                CmdPrepareError::Error(
                    t!("workspace_sheet.error.sheet_name_required_for_delete")
                        .trim()
                        .to_string(),
                )
            })?;
            JVWorkspaceSheetInput::Delete(name.clone())
        }
        (false, false, true, false) => JVWorkspaceSheetInput::ListAll,
        (false, false, false, true) => {
            let name = _args.name.as_ref().ok_or_else(|| {
                CmdPrepareError::Error(
                    t!("workspace_sheet.error.sheet_name_required_for_print_path")
                        .trim()
                        .to_string(),
                )
            })?;
            JVWorkspaceSheetInput::PrintPath(name.clone())
        }
        _ => {
            return Err(CmdPrepareError::Error(
                t!("workspace_sheet.error.exactly_one_required")
                    .trim()
                    .to_string(),
            ));
        }
    };
    Ok(input)
}

async fn collect(_args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    Ok(JVWorkspaceCollect {
        manager: WorkspaceManager::new(),
    })
}

#[exec]
async fn exec(input: In, collect: Collect) -> Result<AnyOutput, CmdExecuteError> {
    match input {
        JVWorkspaceSheetInput::Add(sheet_name) => {
            if let Err(e) = collect.manager.make_sheet(sheet_name).await {
                return Err(MakeSheetErrorConverter::to_exec_error(e));
            }
        }
        JVWorkspaceSheetInput::Delete(sheet_name) => {
            if let Err(e) = collect.manager.drop_sheet(sheet_name).await {
                return Err(MakeSheetErrorConverter::to_exec_error(e));
            }
        }
        JVWorkspaceSheetInput::ListAll => {
            return cmd_output!(JVStringVecOutput => collect
                .manager
                .list_sheet_names().await.into());
        }
        JVWorkspaceSheetInput::PrintPath(sheet_name) => {
            return cmd_output!(JVPathOutput => collect
                .manager
                .get_sheet_path(sheet_name)
                .into());
        }
    }
    cmd_output!(JVNoneOutput => JVNoneOutput)
}

crate::command_template!();
