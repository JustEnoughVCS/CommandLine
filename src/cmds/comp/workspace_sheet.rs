use crate::systems::comp::context::CompletionContext;
use just_enough_vcs::system::workspace::workspace::manager::WorkspaceManager;

pub fn comp(ctx: CompletionContext) -> Option<Vec<String>> {
    if ctx.all_words.len() > 5 {
        return None;
    }

    if ctx.all_words.contains(&"--list-all".to_string())
        || ctx.all_words.contains(&"-A".to_string())
    {
        if ctx.all_words.len() > 4 {
            return None;
        }
    }

    if ctx.current_word.starts_with('-') {
        return Some(vec![
            "--list-all".to_string(),
            "--print-path".to_string(),
            "--new".to_string(),
            "--delete".to_string(),
        ]);
    }

    if ctx.previous_word == "--new" || ctx.previous_word == "-n" {
        return Some(vec![]);
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    Some(rt.block_on(WorkspaceManager::new().list_sheet_names()))
}
