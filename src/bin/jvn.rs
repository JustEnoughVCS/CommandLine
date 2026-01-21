use just_enough_vcs_cli::{subcmd::cmds::_processer::jv_cmd_process, utils::env::current_locales};
use rust_i18n::set_locale;

rust_i18n::i18n!("resources/locales/jv", fallback = "en");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init i18n
    set_locale(&current_locales());

    // Init colored
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    // Collect arguments
    let args: Vec<String> = std::env::args().collect();

    // Process commands
    let render_result = jv_cmd_process(args).await.unwrap_or_default();

    // Print
    print!("{}", render_result);
    Ok(())
}
