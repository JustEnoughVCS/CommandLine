use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct JVSheeteditArgument {
    pub file: Option<PathBuf>,

    #[arg(short, long)]
    pub editor: Option<String>,
}
