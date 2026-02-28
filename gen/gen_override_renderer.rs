use std::{collections::HashSet, path::PathBuf};

use just_template::{Template, tmpl};
use regex::Regex;
use tokio::fs;

use crate::r#gen::{
    constants::{COMMANDS_PATH, OVERRIDE_RENDERER_ENTRY, OVERRIDE_RENDERER_ENTRY_TEMPLATE},
    resolve_types::resolve_type_paths,
};

pub async fn generate_override_renderer(repo_root: &PathBuf) {
    let template_path = repo_root.join(OVERRIDE_RENDERER_ENTRY_TEMPLATE);
    let output_path = repo_root.join(OVERRIDE_RENDERER_ENTRY);
    let all_possible_types = collect_all_possible_types(&PathBuf::from(COMMANDS_PATH)).await;

    // Read the template
    let template_content = tokio::fs::read_to_string(&template_path).await.unwrap();

    // Create template
    let mut template = Template::from(template_content);

    for type_name in &all_possible_types {
        let name = type_name.split("::").last().unwrap_or(type_name);
        tmpl!(template += {
            type_match_arms {
                (jv_output_type_name = name, jv_output_type = type_name)
            }
        });
    }

    // Expand the template
    let final_content = template.expand().unwrap();

    // Write the generated code
    tokio::fs::write(output_path, final_content).await.unwrap();

    println!(
        "Generated override renderer entry with {} types using just_template",
        all_possible_types.len()
    );
}

pub async fn collect_all_possible_types(dir: &PathBuf) -> HashSet<String> {
    let mut all_types = HashSet::new();
    let mut dirs_to_visit = vec![dir.clone()];

    while let Some(current_dir) = dirs_to_visit.pop() {
        let entries_result = fs::read_dir(&current_dir).await;
        if entries_result.is_err() {
            continue;
        }

        let mut entries = entries_result.unwrap();

        loop {
            let entry_result = entries.next_entry().await;
            if entry_result.is_err() {
                break;
            }

            let entry_opt = entry_result.unwrap();
            if entry_opt.is_none() {
                break;
            }

            let entry = entry_opt.unwrap();
            let path = entry.path();

            if path.is_dir() {
                dirs_to_visit.push(path);
                continue;
            }

            let is_rs_file = path.extension().map(|ext| ext == "rs").unwrap_or(false);

            if !is_rs_file {
                continue;
            }

            let code_result = fs::read_to_string(&path).await;
            if code_result.is_err() {
                continue;
            }

            let code = code_result.unwrap();
            let types_opt = resolve_type_paths(&code, get_output_types(&code).unwrap());

            if let Some(types) = types_opt {
                for type_name in types {
                    all_types.insert(type_name);
                }
            }
        }
    }

    all_types
}

pub fn get_output_types(code: &String) -> Option<Vec<String>> {
    let mut output_types = Vec::new();

    // Find all cmd_output! macros
    let cmd_output_re = Regex::new(r"cmd_output!\s*\(\s*([^,]+)\s*=>\s*[^)]+\s*\)").ok()?;
    for cap in cmd_output_re.captures_iter(code) {
        let type_name = cap[1].trim();
        output_types.push(type_name.to_string());
    }

    Some(output_types)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_output_types() {
        const SITUATION: &str = "
        use crate::{
            cmd_output,
            cmds::out::{
                JVCustomOutput, JVCustomOutput2
            },
            systems::cmd::{
                cmd_system::JVCommandContext,
                errors::{CmdExecuteError, CmdPrepareError},
                workspace_reader::LocalWorkspaceReader,
            },
        };
        use cmd_system_macros::exec;
        use other::cmds::output::JVCustomOutputOutside;

        async fn exec() -> Result<(), CmdExecuteError> {
            cmd_output!(output, JVCustomOutput)
            cmd_output!(output, JVCustomOutput2)
            cmd_output!(output, JVCustomOutputNotExist)
            cmd_output!(output, JVCustomOutputOutside)
        }
        ";

        let result = get_output_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let result = result.unwrap();
        let expected = vec![
            "JVCustomOutput".to_string(),
            "JVCustomOutput2".to_string(),
            "JVCustomOutputNotExist".to_string(),
            "JVCustomOutputOutside".to_string(),
        ];
        assert_eq!(result, expected);

        let result = resolve_type_paths(&SITUATION.to_string(), expected);
        assert!(result.is_some(), "Parse failed");
        let result = result.unwrap();
        let expected = vec![
            "crate::cmds::out::JVCustomOutput".to_string(),
            "crate::cmds::out::JVCustomOutput2".to_string(),
            "other::cmds::output::JVCustomOutputOutside".to_string(),
        ];
        assert_eq!(result, expected);
    }
}
