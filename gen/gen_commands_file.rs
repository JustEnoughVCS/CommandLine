use std::path::PathBuf;

use just_fmt::pascal_case;
use just_template::{Template, tmpl, tmpl_param};

use crate::r#gen::constants::{COMMAND_LIST, COMMAND_LIST_TEMPLATE, COMMANDS_PATH, REGISTRY_TOML};

/// Generate registry file from Registry.toml configuration using just_template
pub async fn generate_commands_file(repo_root: &PathBuf) {
    let template_path = repo_root.join(COMMAND_LIST_TEMPLATE);
    let output_path = repo_root.join(COMMAND_LIST);
    let config_path = repo_root.join(REGISTRY_TOML);

    // Read the template
    let template_content = tokio::fs::read_to_string(&template_path).await.unwrap();

    // Read and parse the TOML configuration
    let config_content = tokio::fs::read_to_string(&config_path).await.unwrap();
    let config: toml::Value = toml::from_str(&config_content).unwrap();

    // Collect all command configurations
    let mut all_commands: Vec<(String, String, String)> = Vec::new();
    let mut all_nodes: Vec<String> = Vec::new();

    // Collect commands from registry.toml and COMMANDS_PATH in parallel
    let (registry_collected, auto_collected) = tokio::join!(
        async {
            let mut commands: Vec<(String, String, String)> = Vec::new();
            let mut nodes: Vec<String> = Vec::new();

            let Some(table) = config.as_table() else {
                return (commands, nodes);
            };

            let Some(cmd_table_value) = table.get("cmd") else {
                return (commands, nodes);
            };

            let Some(cmd_table) = cmd_table_value.as_table() else {
                return (commands, nodes);
            };

            for (key, cmd_value) in cmd_table {
                let Some(cmd_config) = cmd_value.as_table() else {
                    continue;
                };

                let Some(node_value) = cmd_config.get("node") else {
                    continue;
                };

                let Some(node_str) = node_value.as_str() else {
                    continue;
                };

                let Some(cmd_type_value) = cmd_config.get("type") else {
                    continue;
                };

                let Some(cmd_type_str) = cmd_type_value.as_str() else {
                    continue;
                };

                let n = node_str.replace(".", " ");
                nodes.push(n.clone());
                commands.push((key.to_string(), n, cmd_type_str.to_string()));
            }

            (commands, nodes)
        },
        async {
            let mut commands: Vec<(String, String, String)> = Vec::new();
            let mut nodes: Vec<String> = Vec::new();
            let commands_dir = repo_root.join(COMMANDS_PATH);
            if commands_dir.exists() && commands_dir.is_dir() {
                let mut entries = tokio::fs::read_dir(&commands_dir).await.unwrap();
                while let Some(entry) = entries.next_entry().await.unwrap() {
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
                    if file_name.starts_with('_') {
                        continue;
                    }

                    // Convert filename to PascalCase
                    let pascal_name = pascal_case!(file_name);

                    let key = file_name.to_string();
                    let node = file_name.replace(".", " ").replace("_", " ");
                    let cmd_type = format!("cmds::cmd::{}::JV{}Command", file_name, pascal_name);

                    nodes.push(node.clone());
                    commands.push((key, node, cmd_type));
                }
            }
            (commands, nodes)
        }
    );

    // Combine the results
    let (mut registry_commands, mut registry_nodes) = registry_collected;
    let (mut auto_commands, mut auto_nodes) = auto_collected;

    all_commands.append(&mut registry_commands);
    all_commands.append(&mut auto_commands);

    all_nodes.append(&mut registry_nodes);
    all_nodes.append(&mut auto_nodes);

    // Create template
    let mut template = Template::from(template_content);

    for (key, node, cmd_type) in &all_commands {
        tmpl!(template += {
            command_match_arms {
                (key = key, node_name = node, cmd_type = cmd_type)
            }
        });
    }

    let nodes_str = format!(
        "[\n        {}\n    ]",
        all_nodes
            .iter()
            .map(|node| format!("\"{}\".to_string()", node))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Use insert_param for the NODES parameter
    tmpl_param!(template, nodes = nodes_str);

    // Expand the template
    let final_content = template.expand().unwrap();

    // Write the generated code
    tokio::fs::write(output_path, final_content).await.unwrap();

    println!(
        "Generated registry file with {} commands using just_template",
        all_commands.len()
    );
}
