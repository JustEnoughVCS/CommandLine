use comp_system_macros::{file_suggest, suggest};
use rust_i18n::t;

use crate::systems::comp::{context::CompletionContext, result::CompletionResult};

pub fn comp(ctx: CompletionContext) -> CompletionResult {
    if ctx.current_word.starts_with('-') {
        return suggest!(
            "-i" = t!("workspace_alias.comp.insert").trim(),
            "--insert" = t!("workspace_alias.comp.insert").trim(),
            "-Q" = t!("workspace_alias.comp.query").trim(),
            "--query" = t!("workspace_alias.comp.query").trim(),
            "-e" = t!("workspace_alias.comp.erase").trim(),
            "--erase" = t!("workspace_alias.comp.erase").trim(),
            "--to" = t!("workspace_alias.comp.to").trim()
        )
        .into();
    }

    if ctx.previous_word == "--to" {
        return suggest!().into();
    }

    file_suggest!()
}
