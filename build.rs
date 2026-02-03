use std::env;
use std::path::PathBuf;

use crate::r#gen::{
    gen_commands_file::generate_commands_file, gen_compile_info::generate_compile_info,
    gen_iscc_script::generate_installer_script, gen_mod_files::generate_collect_files,
    gen_override_renderer::generate_override_renderer, gen_renderers_file::generate_renderers_file,
    gen_specific_renderer::generate_specific_renderer,
};

pub mod r#gen;

#[tokio::main]
async fn main() {
    println!("cargo:rerun-if-env-changed=FORCE_BUILD");

    let repo_root = std::sync::Arc::new(PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()));

    let _ = tokio::join!(
        tokio::spawn({
            let repo_root = repo_root.clone();
            async move { generate_compile_info(&repo_root).await }
        }),
        tokio::spawn({
            let repo_root = repo_root.clone();
            async move { generate_commands_file(&repo_root).await }
        }),
        tokio::spawn({
            let repo_root = repo_root.clone();
            async move { generate_renderers_file(&repo_root).await }
        }),
        tokio::spawn({
            let repo_root = repo_root.clone();
            async move { generate_collect_files(&repo_root).await }
        }),
        tokio::spawn({
            let repo_root = repo_root.clone();
            async move { generate_override_renderer(&repo_root).await }
        }),
        tokio::spawn({
            let repo_root = repo_root.clone();
            async move { generate_specific_renderer(&repo_root).await }
        }),
        tokio::spawn({
            async move {
                if cfg!(target_os = "windows") {
                    // Only generate installer script on Windows
                    let repo_root = repo_root.clone();
                    generate_installer_script(&repo_root).await
                }
            }
        })
    );
}
