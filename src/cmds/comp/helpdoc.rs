use crate::systems::{comp::context::CompletionContext, helpdoc};

pub fn comp(ctx: CompletionContext) -> Option<Vec<String>> {
    if ctx.previous_word == "helpdoc" {
        return Some(
            helpdoc::get_helpdoc_list()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
    }
    None
}
