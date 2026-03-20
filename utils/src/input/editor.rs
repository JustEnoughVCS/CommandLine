use tokio::{fs, process::Command};

use crate::env::editor::get_default_editor;

/// Input text using the system editor
/// Opens the system editor (from EDITOR environment variable) with default text in a cache file,
/// then reads back the modified content after the editor closes, removing comment lines
pub async fn input_with_editor(
    default_text: impl AsRef<str>,
    cache_file: impl AsRef<std::path::Path>,
    comment_char: impl AsRef<str>,
) -> Result<String, std::io::Error> {
    input_with_editor_cutsom(default_text, cache_file, comment_char, get_default_editor()).await
}

pub async fn input_with_editor_cutsom(
    default_text: impl AsRef<str>,
    cache_file: impl AsRef<std::path::Path>,
    comment_char: impl AsRef<str>,
    editor: String,
) -> Result<String, std::io::Error> {
    let cache_path = cache_file.as_ref();
    let default_content = default_text.as_ref();
    let comment_prefix = comment_char.as_ref();

    // Write default text to cache file
    fs::write(cache_path, default_content).await?;

    // Open editor with cache file
    let status = Command::new(editor).arg(cache_path).status().await?;

    if !status.success() {
        return Err(std::io::Error::other("Editor exited with non-zero status"));
    }

    // Read the modified content
    let content = fs::read_to_string(cache_path).await?;

    // Remove comment lines and trim
    let processed_content: String = content
        .lines()
        .filter(|line| !line.trim().starts_with(comment_prefix))
        .collect::<Vec<&str>>()
        .join("\n");

    // Delete the cache file
    let _ = fs::remove_file(cache_path).await;

    Ok(processed_content)
}
