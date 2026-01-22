use serde::Serialize;

use crate::{
    r_print,
    systems::cmd::{
        errors::CmdRenderError,
        renderer::{JVRenderResult, JVResultRenderer},
    },
};

pub struct JVResultPrettyJsonRenderer;

impl<T> JVResultRenderer<T> for JVResultPrettyJsonRenderer
where
    T: Serialize + Sync,
{
    async fn render(data: &T) -> Result<JVRenderResult, CmdRenderError> {
        let mut r = JVRenderResult::default();
        let json_string = serde_json::to_string_pretty(data)
            .map_err(|e| CmdRenderError::SerializeFailed(e.to_string()))?;

        r_print!(r, "{}", json_string);

        Ok(r)
    }
}
