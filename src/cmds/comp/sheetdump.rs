use comp_system_macros::{file_suggest, suggest};
use rust_i18n::t;

use crate::systems::comp::{context::CompletionContext, result::CompletionResult};

pub fn comp(ctx: CompletionContext) -> CompletionResult {
    if ctx.current_word.starts_with('-') {
        return suggest!(
            "--no-sort" = t!("sheetdump.comp.no_sort").trim(),
            "--no-pretty" = t!("sheetdump.comp.no_pretty").trim()
        )
        .into();
    }
    file_suggest!()
}
