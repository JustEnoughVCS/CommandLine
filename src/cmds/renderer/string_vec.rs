use crate::{
    cmds::out::string_vcs::JVStringVecOutput,
    r_println,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};
use render_system_macros::result_renderer;

#[result_renderer(JVStringVecRenderer)]
pub async fn render(data: &JVStringVecOutput) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();
    data.iter().for_each(|s| r_println!(r, "{}", s));
    Ok(r)
}
