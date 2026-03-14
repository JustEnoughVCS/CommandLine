use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CompletionContext {
    /// The full command line
    #[arg(short = 'f', long)]
    command_line: String,

    /// Cursor position
    #[arg(short = 'C', long)]
    cursor_position: usize,

    /// Current word
    #[arg(short = 'w', long)]
    current_word: String,

    /// Previous word
    #[arg(short = 'p', long)]
    previous_word: String,

    /// Command name
    #[arg(short = 'c', long)]
    command_name: String,

    /// Word index
    #[arg(short = 'i', long)]
    word_index: usize,

    /// All words
    #[arg(short = 'a', long, num_args = 1..)]
    all_words: Vec<String>,
}

fn main() {
    let args = match CompletionContext::try_parse() {
        Ok(args) => args,
        Err(_) => std::process::exit(1),
    };
    match comp(args) {
        Some(suggestions) => {
            suggestions
                .iter()
                .for_each(|suggest| println!("{}", suggest));
        }
        None => {
            println!("_file_");
        }
    }
}

fn comp(_args: CompletionContext) -> Option<Vec<String>> {
    None
}
