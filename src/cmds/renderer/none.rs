use render_system_macros::result_renderer;

use crate::{
    cmds::out::none::JVNoneOutput,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

#[result_renderer(JVNoneRenderer)]
pub async fn render(_none: &JVNoneOutput) -> Result<JVRenderResult, CmdRenderError> {
    Ok(JVRenderResult::default())
}
