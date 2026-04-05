#[derive(serde::Serialize)]
pub struct CompileInfo {
    pub date: String,
    pub target: String,
    pub platform: String,
    pub toolchain: String,

    pub cli_version: String,
    pub build_branch: String,
    pub build_commit: String,
}

impl Default for CompileInfo {
    fn default() -> Self {
        Self {
            date: "2026-03-26 16:24:44".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            platform: "Linux".to_string(),
            toolchain: "rustc 1.91.1 (ed61e7d7e 2025-11-07) (stable)".to_string(),
            cli_version: "0.1.1".to_string(),
            build_branch: "dev".to_string(),
            build_commit: "96e19f1b28e9ad3a58864b41a9d6e25ed255dac6".to_string(),
        }
    }
}