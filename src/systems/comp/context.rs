use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CompletionContext {
    /// The full command line
    #[arg(short = 'f', long)]
    pub command_line: String,

    /// Cursor position
    #[arg(short = 'C', long)]
    pub cursor_position: usize,

    /// Current word
    #[arg(short = 'w', long)]
    pub current_word: String,

    /// Previous word
    #[arg(short = 'p', long)]
    pub previous_word: String,

    /// Command name
    #[arg(short = 'c', long)]
    pub command_name: String,

    /// Word index
    #[arg(short = 'i', long)]
    pub word_index: usize,

    /// All words
    #[arg(short = 'a', long, num_args = 1..)]
    pub all_words: Vec<String>,
}
