use crate::env::pager::get_default_pager;
use tokio::{fs, process::Command};

/// Show text using the system pager (less)
/// Opens the system pager (less) with the given text content written to the specified file
/// If less is not found, directly outputs the content to stdout
pub async fn pager(
    content: impl AsRef<str>,
    cache_file: impl AsRef<std::path::Path>,
) -> Result<(), std::io::Error> {
    let content_str = content.as_ref();
    let cache_path = cache_file.as_ref();

    // Write content to cache file
    fs::write(cache_path, content_str).await?;

    // Get the default pager
    let pager_cmd = get_default_pager();

    // Try to use the pager
    let status = Command::new(&pager_cmd).arg(cache_path).status().await;

    match status {
        Ok(status) if status.success() => Ok(()),
        _ => {
            // If pager failed, output directly to stdout
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
