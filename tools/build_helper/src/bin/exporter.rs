use std::{collections::VecDeque, env::current_dir};

use colored::Colorize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("     {} `.cargo/cargo.toml`", "Reading".green().bold());

    let start_time = std::time::Instant::now();

    let target_dir = current_target_dir().expect("Failed to get target directory");
    let publish_dir = current_publish_dir().expect("Failed to get publish directory");
    let publish_binaries = publish_binaries().expect("Failed to get publish binaries");
    let copy_configs = copy_configs().expect("Failed to get copy configurations");

    // Final, export binaries to publish directory
    let copied_files = export(target_dir, publish_dir, publish_binaries, copy_configs)?;

    let duration = start_time.elapsed();
    println!(
        "    {} publish {} {} in {:.1}s",
        "Finished".green().bold(),
        copied_files,
        if copied_files == 1 { "file" } else { "files" },
        duration.as_secs_f32(),
    );

    Ok(())
}

/// Export binaries to publish directory
fn export(
    target_dir: std::path::PathBuf,
    publish_dir: std::path::PathBuf,
    publish_binaries: Vec<String>,
    copy_configs: Vec<CopyConfig>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut copied_files = 0;

    if publish_dir.exists() {
        std::fs::remove_dir_all(&publish_dir)?;
    }
    std::fs::create_dir_all(&publish_dir)?;

    // Create bin directory for binaries
    let bin_dir = publish_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    // Copy binaries to bin directory
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

            if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                && publish_binaries.contains(&file_name.to_string())
            {
                let dest_path = bin_dir.join(file_name);

                if let Some(parent) = dest_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                println!(
                    "      {} `{}` ({})",
                    "Binary".green().bold(),
                    file_name,
                    path.display()
                );
                std::fs::copy(&path, &dest_path)?;
                copied_files += 1;
            }
        }
    }

    // Copy additional files based on configuration
    let current = current_dir()?;
    for config in copy_configs {
        // Check if platforms are specified and if current platform matches
        if !config.platforms.is_empty() {
            let current_platform = std::env::consts::OS;
            if !config.platforms.contains(&current_platform.to_string()) {
                continue;
            }
        }

        let source_path = current.join(&config.from);
        let dest_path = publish_dir.join(&config.to);

        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if source_path.exists() {
            println!(
                "       {} `{}` -> `{}` ({})",
                "Other".green().bold(),
                config.from,
                config.to,
                source_path.display()
            );
            std::fs::copy(&source_path, &dest_path)?;
            copied_files += 1;
        } else {
            println!(
                "     {} `{}` (file not found)",
                "Warning".yellow().bold(),
                config.from
            );
        }
    }

    Ok(copied_files)
}

/// Copy configuration structure
#[derive(Debug)]
struct CopyConfig {
    from: String,
    to: String,
    platforms: Vec<String>,
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

/// Get the copy configurations from the cargo config
/// Returns a vector of CopyConfig structs
fn copy_configs() -> Result<Vec<CopyConfig>, std::io::Error> {
    let current = current_dir()?;
    let config_file = current.join(".cargo").join("config.toml");
    let config_content = std::fs::read_to_string(&config_file)?;
    let config: toml::Value = toml::from_str(&config_content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse config.toml: {}", e),
        )
    })?;

    let mut copy_configs = Vec::new();

    if let Some(copies) = config.get("copies") {
        if let Some(tables) = copies.as_table() {
            for (_, table) in tables {
                if let Some(from) = table.get("from").and_then(|v| v.as_str()) {
                    let to = table.get("to").and_then(|v| v.as_str()).unwrap_or("");

                    // Parse platforms array
                    let mut platforms = Vec::new();
                    if let Some(platforms_array) = table.get("platform").and_then(|v| v.as_array())
                    {
                        for platform in platforms_array {
                            if let Some(platform_str) = platform.as_str() {
                                platforms.push(platform_str.to_string());
                            }
                        }
                    }

                    copy_configs.push(CopyConfig {
                        from: from.to_string(),
                        to: to.to_string(),
                        platforms,
                    });
                }
            }
        }
    }

    Ok(copy_configs)
}

/// Get the target directory of the current project
/// By reading the `build.target-dir` configuration item in the `.cargo/config.toml` file
/// Returns the complete path relative to the current directory
fn current_target_dir() -> Result<std::path::PathBuf, std::io::Error> {
    get_target_dir("build")
}

/// Get the publish directory of the current project
/// By reading the `publish.target-dir` configuration item in the `.cargo/config.toml` file
/// Returns the complete path relative to the current directory
fn current_publish_dir() -> Result<std::path::PathBuf, std::io::Error> {
    get_target_dir("publish")
}

/// Get the binaries list for publishing
// By reading the `publish.binaries` configuration item in the `.cargo/config.toml` file
// Returns a vector of binary names
fn publish_binaries() -> Result<Vec<String>, std::io::Error> {
    get_array("publish", "binaries")
}
