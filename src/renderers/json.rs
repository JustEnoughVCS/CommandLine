use serde::Serialize;
use serde_json;

use crate::{
    r_print,
    systems::cmd::{
        errors::CmdRenderError,
        renderer::{JVRenderResult, JVResultRenderer},
    },
};

pub struct JVResultJsonRenderer;

impl<T> JVResultRenderer<T> for JVResultJsonRenderer
where
    T: Serialize + Sync,
{
    async fn render(data: &T) -> Result<JVRenderResult, CmdRenderError> {
        let mut r = JVRenderResult::default();
        let json_string = serde_json::to_string(data)
            .map_err(|e| CmdRenderError::SerializeFailed(e.to_string()))?;

        r_print!(r, "{}", json_string);

        Ok(r)
    }
}
