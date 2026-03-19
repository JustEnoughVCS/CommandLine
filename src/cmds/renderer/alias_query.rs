use cli_utils::display::markdown::Markdown;
use render_system_macros::result_renderer;
use rust_i18n::t;

use crate::{
    cmds::out::alias_query::JVAliasQueryOutput,
    r_println,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

#[result_renderer(JVAliasQueryRenderer)]
pub async fn render(data: &JVAliasQueryOutput) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();
    match (data.local, data.remote) {
        (local, Some(remote)) => r_println!(
            r,
            "{}",
            t!("workspace_alias.render.map", local = local, remote = remote)
                .trim()
                .markdown()
        ),
        (local, None) => r_println!(
            r,
            "{}",
            t!("workspace_alias.render.no_map", local = local)
                .trim()
                .markdown()
        ),
    }
    Ok(r)
}
