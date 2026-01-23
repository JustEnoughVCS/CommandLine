use std::fmt::{Display, Formatter};

use serde::Serialize;

use crate::systems::cmd::errors::CmdRenderError;

pub trait JVResultRenderer<Data>
where
    Data: Serialize,
{
    fn render(
        data: &Data,
    ) -> impl Future<Output = Result<JVRenderResult, CmdRenderError>> + Send + Sync;
}

#[derive(Default, Debug, PartialEq)]
pub struct JVRenderResult {
    render_text: String,
}

impl Display for JVRenderResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n", self.render_text.trim())
    }
}

impl JVRenderResult {
    pub fn print(&mut self, text: &str) {
        self.render_text.push_str(text);
    }

    pub fn println(&mut self, text: &str) {
        self.render_text.push_str(text);
        self.render_text.push('\n');
    }

    pub fn clear(&mut self) {
        self.render_text.clear();
    }
}

#[macro_export]
macro_rules! r_print {
    ($result:expr, $($arg:tt)*) => {
        $result.print(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! r_println {
    ($result:expr, $($arg:tt)*) => {
        $result.println(&format!($($arg)*))
    };
}
