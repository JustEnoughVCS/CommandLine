use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct JVPathArgument {
    pub path: PathBuf,
}
