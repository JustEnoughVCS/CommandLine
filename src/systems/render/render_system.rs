use std::any::{Any, TypeId};

use crate::systems::{
    cmd::errors::CmdRenderError,
    render::renderer::{JVRenderResult, JVResultRenderer},
};

pub async fn render(
    data: Box<dyn Any + Send + 'static>,
    type_id: TypeId,
) -> Result<JVRenderResult, CmdRenderError> {
    include!("_specific_renderer_matching.rs")
}
