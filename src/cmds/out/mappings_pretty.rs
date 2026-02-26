use just_enough_vcs::system::sheet_system::mapping::LocalMapping;
use serde::Serialize;

#[derive(Serialize)]
pub struct JVMappingsPrettyOutput {
    pub mappings: Vec<LocalMapping>,
}
