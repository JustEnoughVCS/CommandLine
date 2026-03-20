use cli_utils::string_vec;

use crate::systems::comp::context::CompletionContext;

pub fn comp(ctx: CompletionContext) -> Option<Vec<String>> {
    if ctx.all_words.contains(&"--insert".to_string()) {
        if ctx.all_words.len() > 7 {
            return None;
        }
    } else if ctx.all_words.len() > 5 {
        return None;
    }

    if ctx.current_word.starts_with('-') {
        return Some(string_vec![
            "-i", "--insert", "-Q", "--query", "-e", "--erase", "--to",
        ]);
    }

    if ctx.previous_word == "--to" {
        return Some(vec![]);
    }

    None
}
