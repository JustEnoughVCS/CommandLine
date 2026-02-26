use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct JVSheetdumpArgument {
    pub sheet_file: PathBuf,

    #[arg(short, long = "no-sort")]
    pub no_sort: bool,

    #[arg(long = "no-pretty")]
    pub no_pretty: bool,
}
