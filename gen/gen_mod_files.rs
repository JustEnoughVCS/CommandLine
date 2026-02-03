use std::path::PathBuf;

use crate::r#gen::constants::REGISTRY_TOML;

/// Generate collect files from directory structure
pub async fn generate_collect_files(repo_root: &PathBuf) {
    // Read and parse the TOML configuration
    let config_path = repo_root.join(REGISTRY_TOML);
    let config_content = tokio::fs::read_to_string(&config_path).await.unwrap();
    let config: toml::Value = toml::from_str(&config_content).unwrap();

    // Process each collect configuration
    let collect_table = config.get("collect").and_then(|v| v.as_table());

    let collect_table = match collect_table {
        Some(table) => table,
        None => return,
    };

    for (_collect_name, collect_config) in collect_table {
        let config_table = match collect_config.as_table() {
            Some(table) => table,
            None => continue,
        };

        let path_str = match config_table.get("path").and_then(|v| v.as_str()) {
            Some(path) => path,
            None => continue,
        };

        let output_path = repo_root.join(path_str);

        // Extract directory name from the path (e.g., "src/renderers.rs" -> "renderers")
        let dir_name = match output_path.file_stem().and_then(|s| s.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Get the directory path for this collect type
        // e.g., for "src/renderers.rs", we want "src/renderers/"
        let output_parent = output_path.parent().unwrap_or_else(|| repo_root.as_path());
        let dir_path = output_parent.join(&dir_name);

        // Collect all .rs files in the directory (excluding the output file itself)
        let mut modules = Vec::new();

        if dir_path.exists() && dir_path.is_dir() {
            for entry in std::fs::read_dir(&dir_path).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();

                if !path.is_file() {
                    continue;
                }

                let extension = match path.extension() {
                    Some(ext) => ext,
                    None => continue,
                };

                if extension != "rs" {
                    continue;
                }

                let file_name = match path.file_stem().and_then(|s| s.to_str()) {
                    Some(name) => name,
                    None => continue,
                };

                // Skip files that start with underscore
                if !file_name.starts_with('_') {
                    modules.push(file_name.to_string());
                }
            }
        }

        // Sort modules alphabetically
        modules.sort();

        // Generate the content
        let mut content = String::new();
        for module in &modules {
            content.push_str(&format!("pub mod {};\n", module));
        }

        // Write the file
        tokio::fs::write(&output_path, content).await.unwrap();

        println!(
            "Generated {} with {} modules: {:?}",
            path_str,
            modules.len(),
            modules
        );
    }
}
