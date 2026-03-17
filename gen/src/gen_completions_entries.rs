use just_template::{Template, tmpl};
use std::path::PathBuf;

use crate::constants::{COMPLETIONS, COMPLETIONS_PATH, COMPLETIONS_TEMPLATE};

/// Generate completions file from comp directory using just_template
pub async fn generate_completions_file(repo_root: &PathBuf) {
    let template_path = repo_root.join(COMPLETIONS_TEMPLATE);
    let output_path = repo_root.join(COMPLETIONS);
    let comps_dir = repo_root.join(COMPLETIONS_PATH);

    // Read the template
    let template_content = tokio::fs::read_to_string(&template_path).await.unwrap();

    // Collect all completion files
    let mut all_completions: Vec<(String, String)> = Vec::new();
    let mut all_nodes: Vec<String> = Vec::new();

    if comps_dir.exists() && comps_dir.is_dir() {
        let mut entries = tokio::fs::read_dir(&comps_dir).await.unwrap();
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

            let node_name = just_fmt::lower_case!(file_name);

            all_completions.push((file_name.to_string(), node_name.clone()));
            all_nodes.push(node_name);
        }
    }

    // Create template
    let mut template = Template::from(template_content);

    // Generate match arms for each completion
    for (comp_name, node_name) in &all_completions {
        tmpl!(template += {
            comp_match_arms {
                (comp_name = comp_name, comp_node_name = node_name)
            }
        });
    }

    for node in all_nodes {
        tmpl!(template += {
            comp_node_name {
                (comp_node_name = just_fmt::lower_case!(node))
            }
        });
    }

    // Expand the template
    let final_content = template.expand().unwrap();

    // Write the generated code
    tokio::fs::write(output_path, final_content).await.unwrap();

    println!(
        "Generated completions file with {} completions using just_template",
        all_completions.len()
    );
}
