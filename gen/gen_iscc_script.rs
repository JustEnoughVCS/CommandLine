use std::path::PathBuf;

use just_template::{Template, tmpl_param};

use crate::r#gen::{
    constants::{SETUP_JV_CLI_ISS, SETUP_JV_CLI_ISS_TEMPLATE},
    env::{get_author, get_site, get_version},
};

/// Generate Inno Setup installer script (Windows only) using just_template
pub async fn generate_installer_script(repo_root: &PathBuf) {
    let template_path = repo_root.join(SETUP_JV_CLI_ISS_TEMPLATE);
    let output_path = repo_root.join(SETUP_JV_CLI_ISS);

    // Read the template
    let template_content = tokio::fs::read_to_string(&template_path).await.unwrap();

    // Get values for the template
    let author = get_author().unwrap();
    let version = get_version();
    let site = get_site().unwrap();

    // Create template
    let mut template = Template::from(template_content);

    // Set all parameters
    tmpl_param!(template, version = version, author = author, site = site);

    // Expand the template
    let generated = template.expand().unwrap();

    // Write the generated script
    tokio::fs::write(output_path, generated).await.unwrap();

    println!("Generated Inno Setup script using just_template");
}
