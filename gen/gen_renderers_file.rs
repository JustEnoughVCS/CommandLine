use std::path::PathBuf;

use crate::r#gen::constants::{
    OVERRIDE_RENDERER_DISPATCHER, OVERRIDE_RENDERER_DISPATCHER_TEMPLATE, REGISTRY_TOML,
    TEMPLATE_END, TEMPLATE_START,
};

/// Generate renderer list file from Registry.toml configuration
pub async fn generate_renderers_file(repo_root: &PathBuf) {
    let template_path = repo_root.join(OVERRIDE_RENDERER_DISPATCHER_TEMPLATE);
    let output_path = repo_root.join(OVERRIDE_RENDERER_DISPATCHER);
    let config_path = repo_root.join(REGISTRY_TOML);

    // Read the template
    let template = tokio::fs::read_to_string(&template_path).await.unwrap();

    // Read and parse the TOML configuration
    let config_content = tokio::fs::read_to_string(&config_path).await.unwrap();
    let config: toml::Value = toml::from_str(&config_content).unwrap();

    // Collect all renderer configurations
    let mut renderers = Vec::new();

    let Some(table) = config.as_table() else {
        return;
    };
    let Some(renderer_table) = table.get("renderer") else {
        return;
    };
    let Some(renderer_table) = renderer_table.as_table() else {
        return;
    };

    for (_, renderer_value) in renderer_table {
        let Some(renderer_config) = renderer_value.as_table() else {
            continue;
        };
        let Some(name) = renderer_config.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(renderer_type) = renderer_config.get("type").and_then(|v| v.as_str()) else {
            continue;
        };

        renderers.push((name.to_string(), renderer_type.to_string()));
    }

    // Extract the template section from the template content
    const MATCH_MARKER: &str = "// MATCH";

    let template_start_index = template
        .find(TEMPLATE_START)
        .ok_or("Template start marker not found")
        .unwrap();
    let template_end_index = template
        .find(TEMPLATE_END)
        .ok_or("Template end marker not found")
        .unwrap();

    let template_slice = &template[template_start_index..template_end_index + TEMPLATE_END.len()];
    let renderer_template = template_slice
        .trim_start_matches(TEMPLATE_START)
        .trim_end_matches(TEMPLATE_END)
        .trim_matches('\n');

    // Generate the match arms for each renderer
    let match_arms: String = renderers
        .iter()
        .map(|(name, renderer_type)| {
            renderer_template
                .replace("<<NAME>>", name)
                .replace("RendererType", renderer_type)
                .trim_matches('\n')
                .to_string()
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Replace the template section with the generated match arms
    let final_content = template
        .replace(renderer_template, "")
        .replace(TEMPLATE_START, "")
        .replace(TEMPLATE_END, "")
        .replace(MATCH_MARKER, &match_arms)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    // Write the generated code
    tokio::fs::write(output_path, final_content).await.unwrap();

    println!(
        "Generated renderer list file with {} renderers",
        renderers.len()
    );
}
