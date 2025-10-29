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
