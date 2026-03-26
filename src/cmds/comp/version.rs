use comp_system_macros::{file_suggest, suggest};
use rust_i18n::t;

use crate::systems::comp::{context::CompletionContext, result::CompletionResult};

pub fn comp(ctx: CompletionContext) -> CompletionResult {
    if ctx.current_word.starts_with('-') {
        return suggest!(
            "-c" = t!("version.comp.with_compile_info").trim(),
            "--with-compile-info" = t!("version.comp.with_compile_info").trim(),
            "--no-banner" = t!("version.comp.no_banner").trim()
        )
        .into();
    }
    file_suggest!()
}
