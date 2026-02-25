use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct JVSheetdumpArgument {
    pub sheet_file: PathBuf,

    #[arg(short, long)]
    pub sort: bool,
}
