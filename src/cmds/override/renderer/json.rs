use serde::Serialize;
use serde_json;

use crate::{
    r_print,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

pub async fn render<T: Serialize + Send>(data: &T) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();
    let json_string =
        serde_json::to_string(data).map_err(|e| CmdRenderError::SerializeFailed(e.to_string()))?;

    r_print!(r, "{}", json_string);

    Ok(r)
}
