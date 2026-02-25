use serde::Serialize;

#[derive(Serialize)]
pub struct JVHexOutput {
    pub data: Vec<u8>,
}
