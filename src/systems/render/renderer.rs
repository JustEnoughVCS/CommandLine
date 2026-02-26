use std::fmt::{Display, Formatter};
use std::future::Future;
use std::ops::Deref;

use crate::systems::cmd::errors::CmdRenderError;

pub trait JVResultRenderer<Data> {
    fn render(
        data: &Data,
    ) -> impl Future<Output = Result<JVRenderResult, CmdRenderError>> + Send + Sync;

    fn get_type_id(&self) -> std::any::TypeId;
    fn get_data_type_id(&self) -> std::any::TypeId;
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

impl Deref for JVRenderResult {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.render_text
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
