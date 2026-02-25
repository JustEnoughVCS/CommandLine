use serde::Serialize;

use crate::{
    r_print,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

pub async fn render<T: Serialize + Send>(data: &T) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();
    let mut pretty_config = ron::ser::PrettyConfig::new();
    pretty_config.new_line = std::borrow::Cow::from("\n");
    pretty_config.indentor = std::borrow::Cow::from("  ");

    let ron_string = ron::ser::to_string_pretty(data, pretty_config)
        .map_err(|e| CmdRenderError::SerializeFailed(e.to_string()))?;

    r_print!(r, "{}", ron_string);

    Ok(r)
}
