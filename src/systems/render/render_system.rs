use std::any::Any;

use crate::systems::{
    cmd::errors::CmdRenderError,
    render::renderer::{JVRenderResult, JVResultRenderer},
};

pub async fn render(
    data: Box<dyn Any + Send + 'static>,
    type_name: String,
) -> Result<JVRenderResult, CmdRenderError> {
    let type_name_str = type_name.as_str();
    include!("_specific_renderer_matching.rs")
}
