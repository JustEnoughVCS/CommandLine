rust_i18n::i18n!("resources/locales/jvn", fallback = "en");

// --- LIBS ---

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
pub mod args;
pub mod collects;
pub mod inputs;
pub mod outputs;

/// Result Renderers
pub mod renderers;
