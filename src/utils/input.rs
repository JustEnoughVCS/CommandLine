use tokio::{fs, process::Command};

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
    let cache_path = cache_file.as_ref();
    let default_content = default_text.as_ref();
    let comment_prefix = comment_char.as_ref();

    // Write default text to cache file
    fs::write(cache_path, default_content).await?;

    // Get editor from environment variable
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

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
