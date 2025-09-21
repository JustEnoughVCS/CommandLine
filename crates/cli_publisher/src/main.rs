use std::{collections::VecDeque, env::current_dir};

use colored::Colorize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("     {} `.cargo/cargo.toml`", "Reading".green().bold());

    let start_time = std::time::Instant::now();
    let mut copied_files = 0;

    let target_dir = current_target_dir().expect("Failed to get target directory");
    let publish_dir = current_publish_dir().expect("Failed to get publish directory");
    let publish_binaries = publish_binaries().expect("Failed to get publish binaries");

    if publish_dir.exists() {
        std::fs::remove_dir_all(&publish_dir)?;
    }
    std::fs::create_dir_all(&publish_dir)?;

    let mut queue = VecDeque::new();
    queue.push_back(target_dir);

    while let Some(current_dir) = queue.pop_front() {
        let entries = match std::fs::read_dir(&current_dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            if metadata.is_dir() {
                queue.push_back(path);
                continue;
            }

            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if publish_binaries.contains(&file_name.to_string()) {
                    let parent_dir_name = path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("");

                    let dest_path = publish_dir.join(parent_dir_name).join(file_name);

                    if let Some(parent) = dest_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }

                    println!(
                        "        {} `{}/{}` ({})",
                        "Copy".green().bold(),
                        parent_dir_name,
                        file_name,
                        path.display()
                    );
                    std::fs::copy(&path, &dest_path)?;
                    copied_files += 1;
                }
            }
        }
    }

    let duration = start_time.elapsed();
    println!();
    println!(
        "Done (in {:.1}s) Publish {} {}",
        duration.as_secs_f32(),
        copied_files,
        if copied_files == 1 { "file" } else { "files" }
    );

    Ok(())
}

/// Get a target directory from the cargo config
/// Returns the complete path relative to the current directory
fn get_target_dir(section: &str) -> Result<std::path::PathBuf, std::io::Error> {
    let current = current_dir()?;
    let config_file = current.join(".cargo").join("config.toml");
    let config_content = std::fs::read_to_string(&config_file)?;
    let config: toml::Value = toml::from_str(&config_content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse config.toml: {}", e),
        )
    })?;
    let target_dir_str = config[section]["target-dir"].as_str().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "target-dir not found or not a string",
        )
    })?;

    Ok(current.join(target_dir_str))
}

/// Get the binaries list from the cargo config
/// Returns a vector of binary names
fn get_array(section: &str, array_name: &str) -> Result<Vec<String>, std::io::Error> {
    let current = current_dir()?;
    let config_file = current.join(".cargo").join("config.toml");
    let config_content = std::fs::read_to_string(&config_file)?;
    let config: toml::Value = toml::from_str(&config_content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse config.toml: {}", e),
        )
    })?;

    if let Some(array) = config[section][array_name].as_array() {
        let arr: Vec<String> = array
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        Ok(arr)
    } else {
        Ok(Vec::new())
    }
}

/// Get the target directory of the current project
// By reading the `build.target-dir` configuration item in the `.cargo/config.toml` file
// Returns the complete path relative to the current directory
fn current_target_dir() -> Result<std::path::PathBuf, std::io::Error> {
    get_target_dir("build")
}

/// Get the publish directory of the current project
// By reading the `publish.target-dir` configuration item in the `.cargo/config.toml` file
// Returns the complete path relative to the current directory
fn current_publish_dir() -> Result<std::path::PathBuf, std::io::Error> {
    get_target_dir("publish")
}

/// Get the binaries list for publishing
// By reading the `publish.binaries` configuration item in the `.cargo/config.toml` file
// Returns a vector of binary names
fn publish_binaries() -> Result<Vec<String>, std::io::Error> {
    get_array("publish", "binaries")
}
