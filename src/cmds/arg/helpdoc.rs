use clap::Parser;

#[derive(Parser, Debug)]
pub struct JVHelpdocArgument {
    pub doc_name: Option<String>,
}
