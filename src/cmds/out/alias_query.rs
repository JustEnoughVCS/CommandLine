use serde::Serialize;

#[derive(Serialize)]
pub struct JVAliasQueryOutput {
    pub local: u32,
    pub remote: Option<u32>,
}
