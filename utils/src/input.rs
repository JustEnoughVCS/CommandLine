use tokio::{fs, process::Command};

use crate::env::get_default_editor;

/// Confirm the current operation
/// Waits for user input of 'y' or 'n'
pub async fn confirm_hint(text: impl Into<String>) -> bool {
    use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

    let prompt = text.into().trim().to_string();

    let mut stdout = io::stdout();
    let mut stdin = BufReader::new(io::stdin());

    stdout
        .write_all(prompt.as_bytes())
        .await
        .expect("Failed to write prompt");
    stdout.flush().await.expect("Failed to flush stdout");

    let mut input = String::new();
    stdin
        .read_line(&mut input)
        .await
        .expect("Failed to read input");

    input.trim().eq_ignore_ascii_case("y")
}

/// Confirm the current operation, or execute a closure if rejected
/// Waits for user input of 'y' or 'n'
/// If 'n' is entered, executes the provided closure and returns false
pub async fn confirm_hint_or<F>(text: impl Into<String>, on_reject: F) -> bool
where
    F: FnOnce(),
{
    let confirmed = confirm_hint(text).await;
    if !confirmed {
        on_reject();
    }
    confirmed
}

/// Confirm the current operation, and execute a closure if confirmed
/// Waits for user input of 'y' or 'n'
/// If 'y' is entered, executes the provided closure and returns true
pub async fn confirm_hint_then<F>(text: impl Into<String>, on_confirm: F) -> bool
where
    F: FnOnce(),
{
    let confirmed = confirm_hint(text).await;
    if confirmed {
        on_confirm();
    }
    confirmed
}

/// Input text using the system editor
/// Opens the system editor (from EDITOR environment variable) with default text in a cache file,
/// then reads back the modified content after the editor closes, removing comment lines
pub async fn input_with_editor(
    default_text: impl AsRef<str>,
    cache_file: impl AsRef<std::path::Path>,
    comment_char: impl AsRef<str>,
) -> Result<String, std::io::Error> {
    input_with_editor_cutsom(
        default_text,
        cache_file,
        comment_char,
        get_default_editor().await,
    )
    .await
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
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Editor exited with non-zero status",
        ));
    }

    // Read the modified content
    let content = fs::read_to_string(cache_path).await?;

    // Remove comment lines and trim
    let processed_content: String = content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with(comment_prefix) {
                None
            } else {
                Some(line)
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");

    // Delete the cache file
    let _ = fs::remove_file(cache_path).await;

    Ok(processed_content)
}

/// Show text using the system pager (less)
/// Opens the system pager (less) with the given text content written to the specified file
/// If less is not found, directly outputs the content to stdout
pub async fn show_in_pager(
    content: impl AsRef<str>,
    cache_file: impl AsRef<std::path::Path>,
) -> Result<(), std::io::Error> {
    let content_str = content.as_ref();
    let cache_path = cache_file.as_ref();

    // Write content to cache file
    fs::write(cache_path, content_str).await?;

    // Try to use less first
    let status = Command::new("less").arg(cache_path).status().await;

    match status {
        Ok(status) if status.success() => Ok(()),
        _ => {
            // If less failed, output directly to stdout
            use tokio::io::{self, AsyncWriteExt};
            let mut stdout = io::stdout();
            stdout
                .write_all(content_str.as_bytes())
                .await
                .expect("Failed to write content");
            stdout.flush().await.expect("Failed to flush stdout");
            Ok(())
        }
    }
}
