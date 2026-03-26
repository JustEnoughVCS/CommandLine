use std::{fs::OpenOptions, process::exit};

use clap::Parser;
use cli_utils::env::locales::current_locales;
use comp_system_macros::{file_suggest, suggest};
use env_logger::Target;
use jvcli::systems::{
    cmd::_commands::jv_cmd_nodes,
    comp::{
        _comps::{jv_cmd_comp_nodes, match_comp},
        context::{CompletionContext, ShellFlag},
        result::{CompletionResult, CompletionSuggestion},
    },
    render::renderer::jv_override_renderers,
};
#[cfg(debug_assertions)]
use log::debug;
#[cfg(debug_assertions)]
use log::{LevelFilter, error, trace};
use rust_i18n::{set_locale, t};

rust_i18n::i18n!("resources/locales/jvn", fallback = "en");

macro_rules! global_flags_suggest {
    () => {
        suggest!(
            "--confirm" = t!("global_flag.confirm").trim(),
            "-C" = t!("global_flag.confirm").trim(),
            "--help" = t!("global_flag.help").trim(),
            "-h" = t!("global_flag.help").trim(),
            "--lang" = t!("global_flag.lang").trim(),
            "--no-error-logs" = t!("global_flag.no_error_logs").trim(),
            "--no-progress" = t!("global_flag.no_progress").trim(),
            "--quiet" = t!("global_flag.quiet").trim(),
            "-q" = t!("global_flag.quiet").trim(),
            "--renderer" = t!("global_flag.renderer").trim(),
            "--verbose" = t!("global_flag.verbose").trim(),
            "-V" = t!("global_flag.verbose").trim(),
            "--version" = t!("global_flag.version").trim(),
            "-v" = t!("global_flag.version").trim(),
        )
        .into()
    };
}

macro_rules! language_suggest {
    () => {
        // Sort in A - Z order
        suggest!(
            // English
            "en" = "English",
            // Simplified Chinese
            "zh-CN" = "简体中文"
        )
        .into()
    };
}

fn main() {
    // If not in release mode, initialize env_logger to capture logs
    #[cfg(debug_assertions)]
    init_env_logger();

    let lang = current_locales();
    set_locale(&lang);

    // Check if help flag is present in arguments
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!(
            "{}",
            include_str!("../../resources/other/jvn_comp_help.txt").trim()
        );
        std::process::exit(0);
    }

    // Get context parameters from clap
    let ctx = match CompletionContext::try_parse() {
        Ok(args) => {
            CompletionContext {
                // In completion scripts, "-" is replaced with "^", need to convert back here
                command_line: args.command_line.replace('^', "-"),
                cursor_position: args.cursor_position,
                current_word: args.current_word.replace('^', "-"),
                previous_word: args.previous_word.replace('^', "-"),
                command_name: args.command_name.replace('^', "-"),
                word_index: args.word_index,
                all_words: args.all_words.iter().map(|w| w.replace('^', "-")).collect(),
                shell_flag: args.shell_flag,
            }
        }
        Err(e) => {
            // An error occurred, collecting information for output
            #[cfg(debug_assertions)]
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

    #[cfg(debug_assertions)]
    trace!("Generate specific completion");
    let specific_result = specific_comp(&ctx);

    #[cfg(debug_assertions)]
    trace!("Generate default completion");
    let default_result = default_comp(&ctx);

    #[cfg(debug_assertions)]
    trace!("specific_result: {}", specific_result.to_string());
    #[cfg(debug_assertions)]
    trace!("default_result: {}", default_result.to_string());

    let combined_result = match (specific_result, default_result) {
        (CompletionResult::FileCompletion, CompletionResult::FileCompletion) => {
            CompletionResult::file_comp()
        }
        (CompletionResult::Suggestions(s), CompletionResult::FileCompletion) => {
            CompletionResult::Suggestions(s)
        }
        (CompletionResult::FileCompletion, CompletionResult::Suggestions(d)) => {
            CompletionResult::Suggestions(d)
        }
        (CompletionResult::Suggestions(mut s), CompletionResult::Suggestions(d)) => {
            s.extend(d);
            CompletionResult::Suggestions(s)
        }
    };

    handle_comp_result(combined_result, &ctx);
}

fn default_comp(ctx: &CompletionContext) -> CompletionResult {
    if ctx.current_word.starts_with('-') {
        return global_flags_suggest!();
    }

    // Match and comp Override Renderers
    if ctx.previous_word == "--renderer" {
        return jv_override_renderers().into();
    }

    if ctx.previous_word == "--lang" {
        return language_suggest!();
    }

    file_suggest!()
}

fn specific_comp(ctx: &CompletionContext) -> CompletionResult {
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
            #[cfg(debug_assertions)]
            trace!("No matching nodes found, trying command nodes");
            let r = try_comp_cmd_nodes(ctx);
            if r.is_suggestion() {
                #[cfg(debug_assertions)]
                trace!("try_comp_cmd_nodes returned suggestions");
                return r;
            }
            #[cfg(debug_assertions)]
            trace!("try_comp_cmd_nodes returned file completion");
            // No matching node found
            None
        }
        1 => {
            // Single matching node found
            #[cfg(debug_assertions)]
            trace!("Single matching node found: {}", matching_nodes[0]);
            Some(matching_nodes[0].clone())
        }
        _ => {
            // Multiple matching nodes found
            // Find the node with the longest length (most specific match)
            #[cfg(debug_assertions)]
            trace!("Multiple matching nodes found: {:?}", matching_nodes);
            let longest_node = matching_nodes
                .iter()
                .max_by_key(|node| node.len())
                .map(|node| node.to_string());
            #[cfg(debug_assertions)]
            if let Some(ref node) = longest_node {
                trace!("Selected longest node: {}", node);
            }
            longest_node
        }
    };

    #[cfg(debug_assertions)]
    match &match_node {
        Some(node) => trace!("Matched `{}`", node),
        None => trace!("No completions matched."),
    }

    let match_node = match match_node {
        Some(node) => node,
        None => {
            #[cfg(debug_assertions)]
            trace!("No match node found, returning file completion");
            return file_suggest!();
        }
    };

    #[cfg(debug_assertions)]
    trace!("Calling match_comp with node: {}", match_node);
    let result = match_comp(match_node, ctx.clone());
    #[cfg(debug_assertions)]
    trace!("match_comp returned: {}", result.to_string());
    result
}

fn try_comp_cmd_nodes(ctx: &CompletionContext) -> CompletionResult {
    let cmd_nodes = jv_cmd_nodes();

    // If the current position is less than 1, do not perform completion
    if ctx.word_index < 1 {
        return file_suggest!();
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
                return suggestions.into();
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
        file_suggest!()
    } else {
        suggestions.into()
    }
}

fn handle_comp_result(r: CompletionResult, ctx: &CompletionContext) {
    match r {
        CompletionResult::FileCompletion => {
            println!("_file_");
            exit(0)
        }
        CompletionResult::Suggestions(suggestions) => match ctx.shell_flag {
            ShellFlag::Zsh => print_suggest_with_description(suggestions),
            ShellFlag::Fish => print_suggest_with_description_fish(suggestions),
            _ => print_suggest(suggestions),
        },
    }
}

fn print_suggest(mut suggestions: Vec<CompletionSuggestion>) {
    suggestions.sort();
    #[cfg(debug_assertions)]
    trace!("print_suggest suggestions: {:?}", suggestions);
    suggestions
        .iter()
        .for_each(|suggest| println!("{}", suggest.suggest));
    exit(0)
}

fn print_suggest_with_description(mut suggestions: Vec<CompletionSuggestion>) {
    suggestions.sort();
    #[cfg(debug_assertions)]
    trace!(
        "print_suggest_with_description suggestions: {:?}",
        suggestions
    );
    suggestions
        .iter()
        .for_each(|suggest| match &suggest.description {
            Some(desc) => println!("{}$({})", suggest.suggest, desc),
            None => println!("{}", suggest.suggest),
        });
    exit(0)
}

fn print_suggest_with_description_fish(mut suggestions: Vec<CompletionSuggestion>) {
    suggestions.sort();
    #[cfg(debug_assertions)]
    trace!(
        "print_suggest_with_description_fish suggestions: {:?}",
        suggestions
    );
    suggestions
        .iter()
        .for_each(|suggest| match &suggest.description {
            Some(desc) => println!("{}\t{}", suggest.suggest, desc),
            None => println!("{}", suggest.suggest),
        });
    exit(0)
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
    log::trace!("shell_flag={:?}", ctx.shell_flag);
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
