use std::env;
use std::path::PathBuf;

use jv_cli_gen::gen_commands_file::generate_commands_file;
use jv_cli_gen::gen_compile_info::generate_compile_info;
use jv_cli_gen::gen_completions_entries::generate_completions_file;
use jv_cli_gen::gen_iscc_script::generate_installer_script;
use jv_cli_gen::gen_mod_files::generate_collect_files;
use jv_cli_gen::gen_override_renderer::{
    generate_override_renderer, generate_override_renderers_list,
};
use jv_cli_gen::gen_renderers_file::generate_renderers_file;
use jv_cli_gen::gen_specific_renderer::generate_specific_renderer;

#[tokio::main]
async fn main() {
    println!("cargo:rerun-if-env-changed=FORCE_BUILD");
    println!("cargo:rerun-if-changed=src/cmds/arg");
    println!("cargo:rerun-if-changed=src/cmds/cmd");
    println!("cargo:rerun-if-changed=src/cmds/collect");
    println!("cargo:rerun-if-changed=src/cmds/comp");
    println!("cargo:rerun-if-changed=src/cmds/converter");
    println!("cargo:rerun-if-changed=src/cmds/in");
    println!("cargo:rerun-if-changed=src/cmds/out");
    println!("cargo:rerun-if-changed=src/cmds/renderer");

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
            async move { generate_completions_file(&repo_root).await }
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
            async move { generate_override_renderers_list(&repo_root).await }
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
