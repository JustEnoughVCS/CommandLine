use std::env;
use std::path::PathBuf;
use std::process::Command;

use string_proc::pascal_case;

const COMMANDS_PATH: &str = "./src/cmds/";

const COMPILE_INFO_RS_TEMPLATE: &str = "./templates/compile_info.rs.template";
const COMPILE_INFO_RS: &str = "./src/data/compile_info.rs";

const SETUP_JV_CLI_ISS_TEMPLATE: &str = "./templates/setup_jv_cli.iss.template";
const SETUP_JV_CLI_ISS: &str = "./scripts/setup/windows/setup_jv_cli.iss";

const REGISTRY_RS_TEMPLATE: &str = "./templates/_registry.rs.template";
const REGISTRY_RS: &str = "./src/systems/cmd/_registry.rs";

const RENDERER_LIST_TEMPLATE: &str = "./templates/_renderers.rs.template";
const RENDERER_LIST: &str = "./src/systems/cmd/_renderers.rs";

const REGISTRY_TOML: &str = "./.cargo/registry.toml";

fn main() {
    println!("cargo:rerun-if-env-changed=FORCE_BUILD");

    let repo_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    if cfg!(target_os = "windows") {
        // Only generate installer script on Windows
        if let Err(e) = generate_installer_script(&repo_root) {
            eprintln!("Failed to generate installer script: {}", e);
            std::process::exit(1);
        }
    }

    if let Err(e) = generate_compile_info(&repo_root) {
        eprintln!("Failed to generate compile info: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = generate_cmd_registry_file(&repo_root) {
        eprintln!("Failed to generate registry file: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = generate_renderer_list_file(&repo_root) {
        eprintln!("Failed to generate renderer list: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = generate_collect_files(&repo_root) {
        eprintln!("Failed to generate collect files: {}", e);
        std::process::exit(1);
    }
}

/// Generate Inno Setup installer script (Windows only)
fn generate_installer_script(repo_root: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let template_path = repo_root.join(SETUP_JV_CLI_ISS_TEMPLATE);
    let output_path = repo_root.join(SETUP_JV_CLI_ISS);

    let template = std::fs::read_to_string(&template_path)?;

    let author = get_author()?;
    let version = get_version();
    let site = get_site()?;

    let generated = template
        .replace("<<<AUTHOR>>>", &author)
        .replace("<<<VERSION>>>", &version)
        .replace("<<<SITE>>>", &site);

    std::fs::write(output_path, generated)?;
    Ok(())
}

fn get_author() -> Result<String, Box<dyn std::error::Error>> {
    let cargo_toml_path = std::path::Path::new("Cargo.toml");
    let cargo_toml_content = std::fs::read_to_string(cargo_toml_path)?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)?;

    if let Some(package) = cargo_toml.get("package") {
        if let Some(authors) = package.get("authors") {
            if let Some(authors_array) = authors.as_array() {
                if let Some(first_author) = authors_array.get(0) {
                    if let Some(author_str) = first_author.as_str() {
                        return Ok(author_str.to_string());
                    }
                }
            }
        }
    }

    Err("Author not found in Cargo.toml".into())
}

fn get_site() -> Result<String, Box<dyn std::error::Error>> {
    let cargo_toml_path = std::path::Path::new("Cargo.toml");
    let cargo_toml_content = std::fs::read_to_string(cargo_toml_path)?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)?;

    if let Some(package) = cargo_toml.get("package") {
        if let Some(homepage) = package.get("homepage") {
            if let Some(site_str) = homepage.as_str() {
                return Ok(site_str.to_string());
            }
        }
    }

    Err("Homepage not found in Cargo.toml".into())
}

/// Generate compile info
fn generate_compile_info(repo_root: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Read the template code
    let template_code = std::fs::read_to_string(repo_root.join(COMPILE_INFO_RS_TEMPLATE))?;

    let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    let platform = get_platform(&target);
    let toolchain = get_toolchain();
    let version = get_version();
    let branch = get_git_branch().unwrap_or_else(|_| "unknown".to_string());
    let commit = get_git_commit().unwrap_or_else(|_| "unknown".to_string());

    let generated_code = template_code
        .replace("{date}", &date)
        .replace("{target}", &target)
        .replace("{platform}", &platform)
        .replace("{toolchain}", &toolchain)
        .replace("{version}", &version)
        .replace("{branch}", &branch)
        .replace("{commit}", &commit);

    // Write the generated code
    let compile_info_path = repo_root.join(COMPILE_INFO_RS);
    std::fs::write(compile_info_path, generated_code)?;

    Ok(())
}

fn get_platform(target: &str) -> String {
    if target.contains("windows") {
        "Windows".to_string()
    } else if target.contains("linux") {
        "Linux".to_string()
    } else if target.contains("darwin") || target.contains("macos") {
        "macOS".to_string()
    } else if target.contains("android") {
        "Android".to_string()
    } else if target.contains("ios") {
        "iOS".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn get_toolchain() -> String {
    let rustc_version = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string();

    let channel = if rustc_version.contains("nightly") {
        "nightly"
    } else if rustc_version.contains("beta") {
        "beta"
    } else {
        "stable"
    };

    format!("{} ({})", rustc_version, channel)
}

fn get_version() -> String {
    let cargo_toml_path = std::path::Path::new("Cargo.toml");
    let cargo_toml_content = match std::fs::read_to_string(cargo_toml_path) {
        Ok(content) => content,
        Err(_) => return "unknown".to_string(),
    };

    let cargo_toml: toml::Value = match toml::from_str(&cargo_toml_content) {
        Ok(value) => value,
        Err(_) => return "unknown".to_string(),
    };

    if let Some(workspace) = cargo_toml.get("workspace") {
        if let Some(package) = workspace.get("package") {
            if let Some(version) = package.get("version") {
                if let Some(version_str) = version.as_str() {
                    return version_str.to_string();
                }
            }
        }
    }

    "unknown".to_string()
}

/// Get current git branch
fn get_git_branch() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()?;

    if output.status.success() {
        let branch = String::from_utf8(output.stdout)?.trim().to_string();

        if branch.is_empty() {
            // Try to get HEAD reference if no branch (detached HEAD)
            let output = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .output()?;

            if output.status.success() {
                let head_ref = String::from_utf8(output.stdout)?.trim().to_string();
                return Ok(head_ref);
            }
        } else {
            return Ok(branch);
        }
    }

    Err("Failed to get git branch".into())
}

/// Get current git commit hash
fn get_git_commit() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;

    if output.status.success() {
        let commit = String::from_utf8(output.stdout)?.trim().to_string();
        return Ok(commit);
    }

    Err("Failed to get git commit".into())
}

/// Generate registry file from Registry.toml configuration
fn generate_cmd_registry_file(repo_root: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let template_path = repo_root.join(REGISTRY_RS_TEMPLATE);
    let output_path = repo_root.join(REGISTRY_RS);
    let config_path = repo_root.join(REGISTRY_TOML);

    // Read the template
    let template = std::fs::read_to_string(&template_path)?;

    // Read and parse the TOML configuration
    let config_content = std::fs::read_to_string(&config_path)?;
    let config: toml::Value = toml::from_str(&config_content)?;

    // Collect all command configurations
    let mut commands = Vec::new();
    let mut nodes = Vec::new();

    // First, collect commands from Registry.toml
    if let Some(table) = config.as_table() {
        if let Some(cmd_table) = table.get("cmd") {
            if let Some(cmd_table) = cmd_table.as_table() {
                for (key, cmd_value) in cmd_table {
                    if let Some(cmd_config) = cmd_value.as_table() {
                        if let (Some(node), Some(cmd_type)) = (
                            cmd_config.get("node").and_then(|v| v.as_str()),
                            cmd_config.get("type").and_then(|v| v.as_str()),
                        ) {
                            let n = node.replace(".", " ");
                            nodes.push(n.clone());
                            commands.push((key.to_string(), n, cmd_type.to_string()));
                        }
                    }
                }
            }
        }
    }

    // Then, automatically register commands from COMMANDS_PATH
    let commands_dir = repo_root.join(COMMANDS_PATH);
    if commands_dir.exists() && commands_dir.is_dir() {
        for entry in std::fs::read_dir(&commands_dir)? {
            let entry = entry?;
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
            let node = file_name.replace(".", " ");
            let cmd_type = format!("cmds::{}::JV{}Command", file_name, pascal_name);

            nodes.push(node.clone());
            commands.push((key, node, cmd_type));
        }
    }

    // Extract the node_if template from the template content
    const PROCESS_MARKER: &str = "// PROCESS";
    const TEMPLATE_START: &str = "// -- TEMPLATE START --";
    const TEMPLATE_END: &str = "// -- TEMPLATE END --";
    const LINE: &str = "<<LINE>>";
    const NODES: &str = "<<NODES>>";

    let template_start_index = template
        .find(TEMPLATE_START)
        .ok_or("Template start marker not found")?;
    let template_end_index = template
        .find(TEMPLATE_END)
        .ok_or("Template end marker not found")?;

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
    std::fs::write(output_path, final_content)?;

    println!("Generated registry file with {} commands", commands.len());
    Ok(())
}

/// Generate renderer list file from Registry.toml configuration
fn generate_renderer_list_file(repo_root: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let template_path = repo_root.join(RENDERER_LIST_TEMPLATE);
    let output_path = repo_root.join(RENDERER_LIST);
    let config_path = repo_root.join(REGISTRY_TOML);

    // Read the template
    let template = std::fs::read_to_string(&template_path)?;

    // Read and parse the TOML configuration
    let config_content = std::fs::read_to_string(&config_path)?;
    let config: toml::Value = toml::from_str(&config_content)?;

    // Collect all renderer configurations
    let mut renderers = Vec::new();

    if let Some(table) = config.as_table() {
        if let Some(renderer_table) = table.get("renderer") {
            if let Some(renderer_table) = renderer_table.as_table() {
                for (_, renderer_value) in renderer_table {
                    if let Some(renderer_config) = renderer_value.as_table() {
                        if let (Some(name), Some(renderer_type)) = (
                            renderer_config.get("name").and_then(|v| v.as_str()),
                            renderer_config.get("type").and_then(|v| v.as_str()),
                        ) {
                            renderers.push((name.to_string(), renderer_type.to_string()));
                        }
                    }
                }
            }
        }
    }

    // Extract the template section from the template content
    const MATCH_MARKER: &str = "// MATCH";
    const TEMPLATE_START: &str = "// -- TEMPLATE START --";
    const TEMPLATE_END: &str = "// -- TEMPLATE END --";

    let template_start_index = template
        .find(TEMPLATE_START)
        .ok_or("Template start marker not found")?;
    let template_end_index = template
        .find(TEMPLATE_END)
        .ok_or("Template end marker not found")?;

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
    std::fs::write(output_path, final_content)?;

    println!(
        "Generated renderer list file with {} renderers",
        renderers.len()
    );
    Ok(())
}

/// Generate collect files from directory structure
fn generate_collect_files(repo_root: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Read and parse the TOML configuration
    let config_path = repo_root.join(REGISTRY_TOML);
    let config_content = std::fs::read_to_string(&config_path)?;
    let config: toml::Value = toml::from_str(&config_content)?;

    // Process each collect configuration
    let collect_table = config.get("collect").and_then(|v| v.as_table());

    let collect_table = match collect_table {
        Some(table) => table,
        None => return Ok(()),
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
            for entry in std::fs::read_dir(&dir_path)? {
                let entry = entry?;
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
        std::fs::write(&output_path, content)?;

        println!(
            "Generated {} with {} modules: {:?}",
            path_str,
            modules.len(),
            modules
        );
    }

    Ok(())
}
