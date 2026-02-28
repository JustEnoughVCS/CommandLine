use std::path::PathBuf;

use just_template::{Template, tmpl};

use crate::r#gen::constants::{
    OVERRIDE_RENDERER_DISPATCHER, OVERRIDE_RENDERER_DISPATCHER_TEMPLATE, REGISTRY_TOML,
};

/// Generate renderer list file from Registry.toml configuration using just_template
pub async fn generate_renderers_file(repo_root: &PathBuf) {
    let template_path = repo_root.join(OVERRIDE_RENDERER_DISPATCHER_TEMPLATE);
    let output_path = repo_root.join(OVERRIDE_RENDERER_DISPATCHER);
    let config_path = repo_root.join(REGISTRY_TOML);

    // Read the template
    let template_content = tokio::fs::read_to_string(&template_path).await.unwrap();

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

    // Create template
    let mut template = Template::from(template_content);

    for (name, renderer_type) in &renderers {
        tmpl!(template += {
            renderer_match_arms {
                (name = name, renderer_type = renderer_type)
            }
        });
    }

    let final_content = template.expand().unwrap();

    // Write the generated code
    tokio::fs::write(output_path, final_content).await.unwrap();

    println!(
        "Generated renderer list file with {} renderers using just_template",
        renderers.len()
    );
}
