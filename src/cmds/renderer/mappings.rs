use render_system_macros::result_renderer;

use crate::{
    cmds::out::mappings::JVMappingsOutput,
    r_println,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

#[result_renderer(JVMappingsRenderer)]
pub async fn render(data: &JVMappingsOutput) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();
    let mappings = &data.mappings;
    for m in mappings {
        r_println!(r, "{}", m)
    }
    Ok(r)
}
