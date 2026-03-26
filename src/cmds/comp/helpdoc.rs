use comp_system_macros::file_suggest;

use crate::systems::{
    comp::{context::CompletionContext, result::CompletionResult},
    helpdoc,
};

pub fn comp(ctx: CompletionContext) -> CompletionResult {
    if ctx.previous_word == "helpdoc" {
        return helpdoc::get_helpdoc_list()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .into();
    }
    file_suggest!()
}
