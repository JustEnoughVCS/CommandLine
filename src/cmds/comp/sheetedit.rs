use comp_system_macros::{file_suggest, suggest};
use rust_i18n::t;

use crate::systems::comp::{context::CompletionContext, result::CompletionResult};

pub fn comp(ctx: CompletionContext) -> CompletionResult {
    if ctx.current_word.starts_with('-') {
        return suggest!(
            "-e" = t!("sheetedit.comp.editor").trim(),
            "--editor" = t!("sheetedit.comp.editor").trim(),
        )
        .into();
    }

    if ctx.previous_word == "-e" || ctx.previous_word == "--editor" {
        return suggest!().into();
    }

    file_suggest!()
}
