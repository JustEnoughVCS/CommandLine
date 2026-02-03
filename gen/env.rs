use std::process::Command;

pub fn get_author() -> Result<String, Box<dyn std::error::Error>> {
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

pub fn get_site() -> Result<String, Box<dyn std::error::Error>> {
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

pub fn get_platform(target: &str) -> String {
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

pub fn get_toolchain() -> String {
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

pub fn get_version() -> String {
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

pub fn get_git_branch() -> Result<String, Box<dyn std::error::Error>> {
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

pub fn get_git_commit() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;

    if output.status.success() {
        let commit = String::from_utf8(output.stdout)?.trim().to_string();
        return Ok(commit);
    }

    Err("Failed to get git commit".into())
}
