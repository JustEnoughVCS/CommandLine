use std::path::PathBuf;

use string_proc::pascal_case;

use crate::r#gen::constants::{
    COMMAND_LIST, COMMAND_LIST_TEMPLATE, COMMANDS_PATH, REGISTRY_TOML, TEMPLATE_END, TEMPLATE_START,
};

/// Generate registry file from Registry.toml configuration
pub async fn generate_commands_file(repo_root: &PathBuf) {
    let template_path = repo_root.join(COMMAND_LIST_TEMPLATE);
    let output_path = repo_root.join(COMMAND_LIST);
    let config_path = repo_root.join(REGISTRY_TOML);

    // Read the template
    let template = tokio::fs::read_to_string(&template_path).await.unwrap();

    // Read and parse the TOML configuration
    let config_content = tokio::fs::read_to_string(&config_path).await.unwrap();
    let config: toml::Value = toml::from_str(&config_content).unwrap();

    // Collect all command configurations
    let mut commands = Vec::new();
    let mut nodes = Vec::new();

    // Collect commands from registry.toml and COMMANDS_PATH in parallel
    let (registry_collected, auto_collected) = tokio::join!(
        async {
            let mut commands = Vec::new();
            let mut nodes = Vec::new();

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
            let mut commands = Vec::new();
            let mut nodes = Vec::new();
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

    commands.append(&mut registry_commands);
    commands.append(&mut auto_commands);
    nodes.append(&mut registry_nodes);
    nodes.append(&mut auto_nodes);

    // Extract the node_if template from the template content
    const PROCESS_MARKER: &str = "// PROCESS";
    const LINE: &str = "<<LINE>>";
    const NODES: &str = "<<NODES>>";

    let template_start_index = template
        .find(TEMPLATE_START)
        .ok_or("Template start marker not found")
        .unwrap();
    let template_end_index = template
        .find(TEMPLATE_END)
        .ok_or("Template end marker not found")
        .unwrap();

    let template_slice = &template[template_start_index..template_end_index + TEMPLATE_END.len()];
    let node_if_template = template_slice
        .trim_start_matches(TEMPLATE_START)
        .trim_end_matches(TEMPLATE_END)
        .trim_matches('\n');

    // Generate the match arms for each command
    let match_arms: String = commands
        .iter()
        .map(|(key, node, cmd_type)| {
            node_if_template
                .replace("<<KEY>>", key)
                .replace("<<NODE_NAME>>", node)
                .replace("<<COMMAND_TYPE>>", cmd_type)
                .trim_matches('\n')
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");

    let nodes_str = format!(
        "[\n        {}\n    ]",
        nodes
            .iter()
            .map(|node| format!("\"{}\".to_string()", node))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Replace the template section with the generated match arms
    let final_content = template
        .replace(node_if_template, "")
        .replace(TEMPLATE_START, "")
        .replace(TEMPLATE_END, "")
        .replace(PROCESS_MARKER, &match_arms)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .replace(LINE, "")
        .replace(NODES, nodes_str.as_str());

    // Write the generated code
    tokio::fs::write(output_path, final_content).await.unwrap();

    println!("Generated registry file with {} commands", commands.len());
}
