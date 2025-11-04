use std::env;
use std::path::PathBuf;

const COMPILE_INFO_RS: &str = "./src/data/compile_info.rs";
const COMPILE_INFO_RS_TEMPLATE: &str = "./src/data/compile_info.rs.template";

fn main() {
    let repo_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    if let Err(e) = generate_compile_info(&repo_root) {
        eprintln!("Failed to generate compile info: {}", e);
        std::process::exit(1);
    }
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

    let generated_code = template_code
        .replace("{date}", &date)
        .replace("{target}", &target)
        .replace("{platform}", &platform)
        .replace("{toolchain}", &toolchain)
        .replace("{version}", &version);

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
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = PathBuf::from(manifest_dir).join("Cargo.toml");

    if let Ok(contents) = std::fs::read_to_string(&manifest_path) {
        if let Ok(manifest) = contents.parse::<toml::Value>() {
            if let Some(package) = manifest.get("package") {
                if let Some(version) = package.get("version") {
                    if let Some(version_str) = version.as_str() {
                        return version_str.to_string();
                    }
                }
            }
        }
    }

    "0.1.0".to_string()
}
