use crate::data::compile_info::CompileInfo;
use just_enough_vcs::data::compile_info::CoreCompileInfo;

pub struct JVCompileInfoCollect {
    pub compile_info: CompileInfo,
    pub compile_info_core: CoreCompileInfo,
}
