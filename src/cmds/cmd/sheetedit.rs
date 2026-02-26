use crate::{
    cmd_output,
    cmds::{
        arg::sheetedit::JVSheeteditArgument, collect::single_file::JVSingleFileCollect,
        r#in::sheetedit::JVSheeteditInput, out::none::JVNoneOutput,
    },
    systems::cmd::{
        cmd_system::JVCommandContext,
        errors::{CmdExecuteError, CmdPrepareError},
    },
};
use cli_utils::{
    display::SimpleTable, env::get_default_editor, input::input_with_editor_cutsom, string_vec,
};
use cmd_system_macros::exec;
use just_enough_vcs::system::sheet_system::{mapping::LocalMapping, sheet::SheetData};
use just_fmt::fmt_path::{PathFormatError, fmt_path};
use rust_i18n::t;
use std::{any::TypeId, borrow::Cow, path::PathBuf};
use tokio::fs::create_dir_all;

pub struct JVSheeteditCommand;
type Cmd = JVSheeteditCommand;
type Arg = JVSheeteditArgument;
type In = JVSheeteditInput;
type Collect = JVSingleFileCollect;

fn help_str() -> String {
    todo!()
}

async fn prepare(args: &Arg, _ctx: &JVCommandContext) -> Result<In, CmdPrepareError> {
    let file = fmt_path(args.file.clone()).map_err(|e| match e {
        PathFormatError::InvalidUtf8(e) => CmdPrepareError::Error(e.to_string()),
    })?;
    let editor = args.editor.clone().unwrap_or(get_default_editor().await);

    Ok(In { file, editor })
}

async fn collect(args: &Arg, _ctx: &JVCommandContext) -> Result<Collect, CmdPrepareError> {
    let data = match tokio::fs::read(&args.file).await {
        Ok(d) => d,
        Err(_) => Vec::new(),
    };
    Ok(Collect { data })
}

#[exec]
async fn exec(
    input: In,
    collect: Collect,
) -> Result<(Box<dyn std::any::Any + Send + 'static>, TypeId), CmdExecuteError> {
    let sheet = SheetData::try_from(collect.data).unwrap_or(SheetData::empty());

    let mappings = sheet.mappings();
    let mut mappings_vec = mappings.iter().cloned().collect::<Vec<LocalMapping>>();
    mappings_vec.sort();

    let template = build_template(&input.file, mappings_vec).to_string();

    let temp_file = input.file.with_added_extension("md");
    create_dir_all(temp_file.parent().unwrap()).await?;

    let edit_result = input_with_editor_cutsom(template, &temp_file, "#", input.editor).await;

    match edit_result {
        Ok(t) => {
            let rebuild_sheet_data = SheetData::try_from(t.as_str())
                .map_err(|e| CmdExecuteError::Error(e.to_string()))?;
            tokio::fs::write(&input.file, rebuild_sheet_data.as_bytes()).await?;
        }
        Err(e) => return Err(CmdExecuteError::Error(e.to_string())),
    }

    cmd_output!(JVNoneOutput => JVNoneOutput {})
}

fn build_template(file: &PathBuf, mappings: Vec<LocalMapping>) -> Cow<'static, str> {
    let mapping_table = render_pretty_mappings(&mappings);
    let template = t!(
        "sheetedit.editor",
        file_dir = file.display(),
        info = mapping_table
    );

    template
}

fn render_pretty_mappings(mappings: &Vec<LocalMapping>) -> String {
    let header = string_vec![
        format!("#   {}", t!("sheetedit.mapping")),
        "",
        t!("sheetedit.index_source"),
        "",
        t!("sheetedit.forward")
    ];

    let mut simple_table = SimpleTable::new(header);

    for mapping in mappings {
        let mapping_str = mapping
            .to_string()
            .split(" ")
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        simple_table.push_item(vec![
            format!(
                "    {}      ",
                mapping_str.get(0).unwrap_or(&String::default())
            ), // Mapping
            format!("{} ", mapping_str.get(1).unwrap_or(&String::default())), // => & ==
            format!("{}      ", mapping_str.get(2).unwrap_or(&String::default())), // Index
            format!("{} ", mapping_str.get(3).unwrap_or(&String::default())), // => & ==
            format!("{}      ", mapping_str.get(4).unwrap_or(&String::default())), // Forward
        ]);
    }
    simple_table.to_string()
}

crate::command_template!();
