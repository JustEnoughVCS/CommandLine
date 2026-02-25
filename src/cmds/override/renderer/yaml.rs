use serde::Serialize;

use crate::{
    r_print,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

pub async fn render<T: Serialize + Send>(data: &T) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();
    let yaml_string =
        serde_yaml::to_string(data).map_err(|e| CmdRenderError::SerializeFailed(e.to_string()))?;

    r_print!(r, "{}", yaml_string);

    Ok(r)
}
