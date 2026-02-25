use serde::Serialize;

use crate::{
    r_print,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

pub async fn render<T: Serialize + Send>(data: &T) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();

    let ron_string =
        ron::ser::to_string(data).map_err(|e| CmdRenderError::SerializeFailed(e.to_string()))?;

    r_print!(r, "{}", ron_string);

    Ok(r)
}
