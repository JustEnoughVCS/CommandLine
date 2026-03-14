use crate::data::compile_info::CompileInfo;
use just_enough_vcs::data::compile_info::CoreCompileInfo;

#[derive(serde::Serialize)]
pub struct JVVersionOutput {
    pub show_compile_info: bool,
    pub show_banner: bool,
    pub compile_info: CompileInfo,
    pub compile_info_core: CoreCompileInfo,
}
