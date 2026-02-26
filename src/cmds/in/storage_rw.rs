use std::path::PathBuf;

use just_enough_vcs::system::storage_system::store::ChunkingPolicy;

pub struct JVStorageRWInput {
    pub input: PathBuf,
    pub storage: PathBuf,
    pub output: PathBuf,

    pub chunking_policy: Option<ChunkingPolicy>,
}
