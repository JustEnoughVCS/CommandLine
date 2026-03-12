pub async fn move_across_partitions(
    source_path: impl AsRef<std::path::Path>,
    dest_path: impl AsRef<std::path::Path>,
) -> Result<(), std::io::Error> {
    let source_path = source_path.as_ref();
    let dest_path = dest_path.as_ref();
    if !source_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Source file does not exist",
        ));
    }

    if let Ok(()) = std::fs::rename(source_path, dest_path) {
        return Ok(());
    }

    std::fs::copy(source_path, dest_path)?;
    std::fs::remove_file(source_path)?;

    Ok(())
}

pub async fn copy_across_partitions(
    source_path: impl AsRef<std::path::Path>,
    dest_path: impl AsRef<std::path::Path>,
) -> Result<(), std::io::Error> {
    let source_path = source_path.as_ref();
    let dest_path = dest_path.as_ref();
    if !source_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Source file does not exist",
        ));
    }

    std::fs::copy(source_path, dest_path)?;

    Ok(())
}
