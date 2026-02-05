use clap::Parser;

#[derive(Parser, Debug)]
pub struct JVStatusArgument {
    what: Option<String>, // status of what to query
}
