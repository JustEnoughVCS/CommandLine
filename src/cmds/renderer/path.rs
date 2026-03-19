use crate::{
    cmds::out::path::JVPathOutput,
    r_println,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};
use render_system_macros::result_renderer;

#[result_renderer(JVPathRenderer)]
pub async fn render(data: &JVPathOutput) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();
    r_println!(r, "{}", data.display());
    Ok(r)
}
