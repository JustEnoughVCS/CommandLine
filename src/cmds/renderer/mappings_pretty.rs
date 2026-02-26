use cli_utils::{display::SimpleTable, string_vec};
use colored::Colorize;
use just_enough_vcs::system::sheet_system::mapping::LocalMapping;
use render_system_macros::result_renderer;
use rust_i18n::t;

use crate::{
    cmds::out::mappings_pretty::JVMappingsPrettyOutput,
    r_println,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

#[result_renderer(JVMappingsPrettyRenderer)]
pub async fn render(data: &JVMappingsPrettyOutput) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();
    let mappings = &data.mappings;
    r_println!(r, "{}", render_pretty_mappings(&mappings));
    Ok(r)
}

fn render_pretty_mappings(mappings: &Vec<LocalMapping>) -> String {
    let header = string_vec![
        "0",
        format!(" | {}", t!("sheetedit.mapping").bold()),
        "",
        format!("| {}", t!("sheetedit.index_source").bold()),
        "",
        format!("| {}", t!("sheetedit.forward").bold()),
        "|"
    ];

    let mut simple_table = SimpleTable::new(header);

    let mut i = 1;
    for mapping in mappings {
        let mapping_str = mapping
            .to_string()
            .split(" ")
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        simple_table.push_item(vec![
            // Number
            format!("{}", i).bold().to_string(),
            // Mapping
            format!(
                " | {} ",
                mapping_str
                    .get(0)
                    .unwrap_or(&String::default())
                    .bright_cyan()
            ),
            // => & ==
            format!(
                "{} ",
                match mapping_str.get(1).unwrap_or(&String::default()).as_str() {
                    "==" => mapping_str[1].bright_yellow().bold(),
                    "=>" => mapping_str[1].bright_yellow(),
                    _ => mapping_str[1].bright_black(),
                }
            ),
            // Index
            format!(
                " {} ",
                mapping_str
                    .get(2)
                    .unwrap_or(&String::default())
                    .bright_yellow()
            ),
            // => & ==
            format!(
                "{} ",
                match mapping_str.get(1).unwrap_or(&String::default()).as_str() {
                    "==" => mapping_str[1].bright_yellow().bold(),
                    "=>" => mapping_str[1].bright_yellow(),
                    _ => mapping_str[1].bright_black(),
                }
            ),
            // Forward
            format!(
                " {} ",
                mapping_str
                    .get(4)
                    .unwrap_or(&String::default())
                    .bright_yellow()
            ),
            "|".to_string(),
        ]);

        i += 1;
    }
    simple_table.to_string()
}
