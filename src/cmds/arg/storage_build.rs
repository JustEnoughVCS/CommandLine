use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct JVStorageBuildArgument {
    pub index_file: PathBuf,
    pub storage: PathBuf,

    #[arg(short = 'o', long = "output")]
    pub output_file: Option<PathBuf>,
}
