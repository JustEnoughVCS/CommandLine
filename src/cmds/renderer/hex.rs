use just_enough_vcs::utils::hex_display::hex_display_vec;
use render_system_macros::result_renderer;

use crate::{
    cmds::out::hex::JVHexOutput,
    r_println,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

#[result_renderer(JVHexRenderer)]
pub async fn render(data: &JVHexOutput) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();
    r_println!(r, "{}", hex_display_vec(data.data.clone()));
    Ok(r)
}
