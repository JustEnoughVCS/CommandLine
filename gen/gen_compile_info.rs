use std::path::PathBuf;

use crate::r#gen::{
    constants::{COMPILE_INFO_RS, COMPILE_INFO_RS_TEMPLATE},
    env::{get_git_branch, get_git_commit, get_platform, get_toolchain, get_version},
};

/// Generate compile info
pub async fn generate_compile_info(repo_root: &PathBuf) {
    // Read the template code
    let template_code = tokio::fs::read_to_string(repo_root.join(COMPILE_INFO_RS_TEMPLATE))
        .await
        .unwrap();

    let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
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
    tokio::fs::write(compile_info_path, generated_code)
        .await
        .unwrap();
}
