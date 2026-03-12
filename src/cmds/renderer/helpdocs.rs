use render_system_macros::result_renderer;

use crate::{
    cmds::out::helpdocs::JVHelpdocsOutput,
    r_println,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

#[result_renderer(JVHelpdocsRenderer)]
pub async fn render(data: &JVHelpdocsOutput) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();
    data.names.iter().for_each(|d| r_println!(r, "{}", d));
    Ok(r)
}
