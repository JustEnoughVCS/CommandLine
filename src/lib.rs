rust_i18n::i18n!("resources/locales/jvn", fallback = "en");

// --- LIBS ---

/// Utils
pub mod utils;

/// Data
pub mod data;

/// Json Format
pub mod legacy_json_output;

/// Systems
pub mod systems;

// --- ASSETS ---

/// Commands
pub mod cmds;

/// Command Data
pub mod arguments;
pub mod inputs;
pub mod outputs;

/// Result Renderers
pub mod renderers;
