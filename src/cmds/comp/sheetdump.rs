use cli_utils::string_vec;

use crate::systems::comp::context::CompletionContext;

pub fn comp(ctx: CompletionContext) -> Option<Vec<String>> {
    if ctx.current_word.starts_with('-') {
        return Some(string_vec!["--no-sort", "--no-pretty"]);
    }

    None
}
