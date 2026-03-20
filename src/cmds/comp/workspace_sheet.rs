use crate::systems::comp::context::CompletionContext;
use cli_utils::string_vec;
use just_enough_vcs::system::workspace::workspace::manager::WorkspaceManager;

pub fn comp(ctx: CompletionContext) -> Option<Vec<String>> {
    if ctx.all_words.len() > 5 {
        return None;
    }

    if (ctx.all_words.contains(&"--list-all".to_string())
        || ctx.all_words.contains(&"-A".to_string()))
        && ctx.all_words.len() > 4 {
            return None;
        }

    if ctx.current_word.starts_with('-') {
        return Some(string_vec![
            "-A",
            "--list-all",
            "-p",
            "--print-path",
            "-n",
            "--new",
            "-d",
            "--delete",
        ]);
    }

    if ctx.previous_word == "--new" || ctx.previous_word == "-n" {
        return Some(vec![]);
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    Some(rt.block_on(WorkspaceManager::new().list_sheet_names()))
}
