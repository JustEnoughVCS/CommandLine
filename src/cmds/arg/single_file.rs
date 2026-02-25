use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct JVSingleFileArgument {
    pub file: PathBuf,
}
