use std::path::PathBuf;

use crate::r#gen::{
    constants::{SETUP_JV_CLI_ISS, SETUP_JV_CLI_ISS_TEMPLATE},
    env::{get_author, get_site, get_version},
};

/// Generate Inno Setup installer script (Windows only)
pub async fn generate_installer_script(repo_root: &PathBuf) {
    let template_path = repo_root.join(SETUP_JV_CLI_ISS_TEMPLATE);
    let output_path = repo_root.join(SETUP_JV_CLI_ISS);

    let template = tokio::fs::read_to_string(&template_path).await.unwrap();

    let author = get_author().unwrap();
    let version = get_version();
    let site = get_site().unwrap();

    let generated = template
        .replace("<<<AUTHOR>>>", &author)
        .replace("<<<VERSION>>>", &version)
        .replace("<<<SITE>>>", &site);

    tokio::fs::write(output_path, generated).await.unwrap();
}
