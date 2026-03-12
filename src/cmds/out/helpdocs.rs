use serde::Serialize;

#[derive(Serialize)]
pub struct JVHelpdocsOutput {
    pub names: Vec<String>,
}
