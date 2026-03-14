use crate::cmds::{
    collect::compile_info::JVCompileInfoCollect, r#in::version::JVVersionInput,
    out::version::JVVersionOutput,
};

pub struct JVVersionInputOutputConverter;

impl JVVersionInputOutputConverter {
    pub fn merge_to_output(
        input: JVVersionInput,
        collect: JVCompileInfoCollect,
    ) -> JVVersionOutput {
        JVVersionOutput {
            show_compile_info: input.show_compile_info,
            show_banner: input.show_banner,
            compile_info: collect.compile_info,
            compile_info_core: collect.compile_info_core,
        }
    }
}
