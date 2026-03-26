use comp_system_macros::{file_suggest, suggest};
use just_enough_vcs::system::workspace::workspace::manager::WorkspaceManager;
use rust_i18n::t;

use crate::systems::comp::{context::CompletionContext, result::CompletionResult};

pub fn comp(ctx: CompletionContext) -> CompletionResult {
    if ctx.current_word.starts_with('-') {
        return suggest!(
            "-A" = t!("workspace_sheet.comp.list_all").trim(),
            "--list-all" = t!("workspace_sheet.comp.list_all").trim(),
            "-p" = t!("workspace_sheet.comp.print_path").trim(),
            "--print-path" = t!("workspace_sheet.comp.print_path").trim(),
            "-n" = t!("workspace_sheet.comp.new").trim(),
            "--new" = t!("workspace_sheet.comp.new").trim(),
            "-d" = t!("workspace_sheet.comp.delete").trim(),
            "--delete" = t!("workspace_sheet.comp.delete").trim()
        )
        .into();
    }

    if ctx.previous_word == "--new" || ctx.previous_word == "-n" {
        return suggest!().into();
    }

    if ctx.previous_word == "--list-all"
        || ctx.previous_word == "-A"
        || ctx.previous_word == "--print-path"
        || ctx.previous_word == "-p"
        || ctx.previous_word == "--delete"
        || ctx.previous_word == "-d"
    {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let names = rt.block_on(WorkspaceManager::new().list_sheet_names());
        return names.into();
    }

    file_suggest!()
}
