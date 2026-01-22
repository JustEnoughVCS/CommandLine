use crate::{
    outputs::status::JVStatusResult,
    r_println,
    systems::cmd::{
        errors::CmdRenderError,
        renderer::{JVRenderResult, JVResultRenderer},
    },
};

pub struct JVStatusRenderer;

impl JVResultRenderer<JVStatusResult> for JVStatusRenderer {
    async fn render(_data: &JVStatusResult) -> Result<JVRenderResult, CmdRenderError> {
        let mut r = JVRenderResult::default();
        r_println!(r, "Nothing");
        Ok(r)
    }
}
