use std::{fs::OpenOptions, process::exit};

use clap::Parser;
use env_logger::Target;
use jvcli::systems::{
    cmd::_commands::jv_cmd_nodes,
    comp::{
        _comps::{jv_cmd_comp_nodes, match_comp},
        context::CompletionContext,
    },
    render::renderer::jv_override_renderers,
};
#[cfg(debug_assertions)]
use log::debug;
use log::{LevelFilter, error, trace};

const GLOBAL_FLAGS: &[&str] = &[
    "--confirm",
    "-C",
    "--help",
    "-h",
    "--lang",
    "--no-error-logs",
    "--no-progress",
    "--quiet",
    "-q",
    "--renderer",
    "--verbose",
    "-V",
    "--version",
    "-v",
];

const LANGUAGES: [&str; 2] = ["en", "zh-CN"];

fn main() {
    // If not in release mode, initialize env_logger to capture logs
    #[cfg(debug_assertions)]
    init_env_logger();

    // Get context parameters from clap
    let ctx = match CompletionContext::try_parse() {
        Ok(args) => CompletionContext {
            // In completion scripts, "-" is replaced with "^", need to convert back here
            command_line: args.command_line.replace('^', "-"),
            cursor_position: args.cursor_position,
            current_word: args.current_word.replace('^', "-"),
            previous_word: args.previous_word.replace('^', "-"),
            command_name: args.command_name.replace('^', "-"),
            word_index: args.word_index,
            all_words: args.all_words.iter().map(|w| w.replace('^', "-")).collect(),
        },
        Err(e) => {
            // An error occurred, collecting information for output
            error!(
                "Error: {}, origin=\"{}\"",
                e,
                std::env::args().collect::<Vec<String>>().join(" ")
            );
            std::process::exit(1);
        }
    };

    // Trace context information
    #[cfg(debug_assertions)]
    trace_ctx(&ctx);

    trace!("Try using specific completion");
    let result = comp(&ctx);
    if let Some(suggestions) = result {
        handle_comp_result(&Some(suggestions));
    } else {
        trace!("Using default completion");
        let result = default_comp(&ctx);
        handle_comp_result(&result);
    }
}

fn default_comp(ctx: &CompletionContext) -> Option<Vec<String>> {
    if ctx.current_word.starts_with('-') {
        return Some(GLOBAL_FLAGS.iter().map(|s| s.to_string()).collect());
    }

    // Match and comp Override Renderers
    if ctx.previous_word == "--renderer" {
        return Some(jv_override_renderers());
    }

    if ctx.previous_word == "--lang" {
        return Some(LANGUAGES.iter().map(|s| s.to_string()).collect());
    }

    None
}

fn comp(ctx: &CompletionContext) -> Option<Vec<String>> {
    let args: Vec<String> = ctx.all_words.iter().skip(1).cloned().collect();
    let nodes = jv_cmd_comp_nodes();
    let command = format!("{} ", args.join(" "));

    #[cfg(debug_assertions)]
    debug!("Arguments: `{}`", command);

    // Find all nodes that match the command prefix
    let matching_nodes: Vec<&String> = nodes
        .iter()
        .filter(|node| {
            let matches = command.starts_with(&format!("{} ", node));
            #[cfg(debug_assertions)]
            debug!("Checking node '{}': matches = {}", node, matches);
            matches
        })
        .collect();

    let match_node: Option<String> = match matching_nodes.len() {
        0 => {
            if let Some(result) = try_comp_cmd_nodes(ctx) {
                return Some(result);
            }
            // No matching node found
            None
        }
        1 => {
            // Single matching node found
            Some(matching_nodes[0].clone())
        }
        _ => {
            // Multiple matching nodes found
            // Find the node with the longest length (most specific match)
            matching_nodes
                .iter()
                .max_by_key(|node| node.len())
                .map(|node| node.to_string())
        }
    };

    #[cfg(debug_assertions)]
    match &match_node {
        Some(node) => trace!("Matched `{}`", node),
        None => trace!("No completions matched."),
    }

    let match_node = match_node?;

    match_comp(match_node, ctx.clone())
}

fn try_comp_cmd_nodes(ctx: &CompletionContext) -> Option<Vec<String>> {
    let cmd_nodes = jv_cmd_nodes();

    // If the current position is less than 1, do not perform completion
    if ctx.word_index < 1 {
        return None;
    };

    // Get the current input path
    let input_path: Vec<&str> = ctx.all_words[1..ctx.word_index]
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.as_str())
        .collect();

    #[cfg(debug_assertions)]
    debug!(
        "try_comp_cmd_nodes: input_path = {:?}, word_index = {}, all_words = {:?}",
        input_path, ctx.word_index, ctx.all_words
    );

    // Filter command nodes that match the input path
    let mut suggestions = Vec::new();

    // Special case: if input_path is empty, return all first-level commands
    if input_path.is_empty() {
        for node in cmd_nodes {
            let node_parts: Vec<&str> = node.split(' ').collect();
            if !node_parts.is_empty() && !suggestions.contains(&node_parts[0].to_string()) {
                suggestions.push(node_parts[0].to_string());
            }
        }
    } else {
        // Get the current word
        let current_word = input_path.last().unwrap();

        // First, handle partial match completion for the current word
        // Only perform current word completion when current_word is not empty
        if input_path.len() == 1 && !ctx.current_word.is_empty() {
            for node in &cmd_nodes {
                let node_parts: Vec<&str> = node.split(' ').collect();
                if !node_parts.is_empty()
                    && node_parts[0].starts_with(current_word)
                    && !suggestions.contains(&node_parts[0].to_string())
                {
                    suggestions.push(node_parts[0].to_string());
                }
            }

            // If suggestions for the current word are found, return directly
            if !suggestions.is_empty() {
                suggestions.sort();
                suggestions.dedup();
                #[cfg(debug_assertions)]
                debug!(
                    "try_comp_cmd_nodes: current word suggestions = {:?}",
                    suggestions
                );
                return Some(suggestions);
            }
        }

        // Handle next-level command suggestions
        for node in cmd_nodes {
            let node_parts: Vec<&str> = node.split(' ').collect();

            #[cfg(debug_assertions)]
            debug!("Checking node: '{}', parts: {:?}", node, node_parts);

            // If input path is longer than node parts, skip
            if input_path.len() > node_parts.len() {
                continue;
            }

            // Check if input path matches the beginning of node parts
            let mut matches = true;
            for i in 0..input_path.len() {
                if i >= node_parts.len() {
                    matches = false;
                    break;
                }

                if i == input_path.len() - 1 {
                    if !node_parts[i].starts_with(input_path[i]) {
                        matches = false;
                        break;
                    }
                } else if input_path[i] != node_parts[i] {
                    matches = false;
                    break;
                }
            }

            if matches && input_path.len() <= node_parts.len() {
                if input_path.len() == node_parts.len() && !ctx.current_word.is_empty() {
                    suggestions.push(node_parts[input_path.len() - 1].to_string());
                } else if input_path.len() < node_parts.len() {
                    suggestions.push(node_parts[input_path.len()].to_string());
                }
            }
        }
    }

    // Remove duplicates and sort
    suggestions.sort();
    suggestions.dedup();

    #[cfg(debug_assertions)]
    debug!("try_comp_cmd_nodes: suggestions = {:?}", suggestions);

    if suggestions.is_empty() {
        None
    } else {
        Some(suggestions)
    }
}

fn handle_comp_result(r: &Option<Vec<String>>) {
    match r {
        Some(suggestions) => {
            suggestions
                .iter()
                .for_each(|suggest| println!("{}", suggest));
            exit(0)
        }
        None => {
            // Output "_file_" to notify the completion script to perform "file completion"
            println!("_file_");
            exit(0)
        }
    }
}

#[cfg(debug_assertions)]
fn trace_ctx(ctx: &CompletionContext) {
    log::trace!("command_line={}", ctx.command_line);
    log::trace!("cursor_position={}", ctx.cursor_position);
    log::trace!("current_word={}", ctx.current_word);
    log::trace!("previous_word={}", ctx.previous_word);
    log::trace!("command_name={}", ctx.command_name);
    log::trace!("word_index={}", ctx.word_index);
    log::trace!("all_words={:?}", ctx.all_words);
}

#[cfg(debug_assertions)]
fn init_env_logger() {
    let mut log_path = std::env::current_exe()
        .expect("Failed to get current executable path")
        .parent()
        .expect("Failed to get parent directory")
        .to_path_buf();
    log_path.push("../../jv_comp_log.txt");

    // Only initialize logger if log file exists
    if log_path.exists() {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("Failed to open log file");

        env_logger::Builder::new()
            .filter_level(LevelFilter::Trace)
            .target(Target::Pipe(Box::new(log_file)))
            .init();
    }
}
