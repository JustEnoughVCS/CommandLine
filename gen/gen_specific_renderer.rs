use std::{collections::HashMap, path::PathBuf};

use just_template::{Template, tmpl};
use regex::Regex;

use crate::r#gen::{
    constants::{RENDERERS_PATH, SPECIFIC_RENDERER_MATCHING, SPECIFIC_RENDERER_MATCHING_TEMPLATE},
    resolve_types::resolve_type_paths,
};

const RENDERER_TYPE_PREFIX: &str = "crate::";

/// Generate specific renderer matching file using just_template
pub async fn generate_specific_renderer(repo_root: &PathBuf) {
    // Matches: HashMap<RendererTypeFullName, OutputTypeFullName>
    let mut renderer_matches: HashMap<String, String> = HashMap::new();

    let renderer_path = repo_root.join(RENDERERS_PATH);
    collect_renderers(&renderer_path, &mut renderer_matches);

    let template_path = repo_root.join(SPECIFIC_RENDERER_MATCHING_TEMPLATE);
    let output_path = repo_root.join(SPECIFIC_RENDERER_MATCHING);

    // Read the template
    let template_content = tokio::fs::read_to_string(&template_path).await.unwrap();

    // Create template
    let mut template = Template::from(template_content);

    for (renderer, output) in &renderer_matches {
        let output_name = output.split("::").last().unwrap_or(output);
        tmpl!(template += {
            renderer_match_arms {
                (output_type_name = output_name, output_type = output, renderer_type = renderer)
            }
        });
    }

    let final_content = template.expand().unwrap();

    // Write the generated code
    tokio::fs::write(output_path, final_content).await.unwrap();

    println!(
        "Generated specific renderer matching with {} renderers using just_template",
        renderer_matches.len()
    );
}

fn collect_renderers(dir_path: &PathBuf, matches: &mut HashMap<String, String>) {
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    collect_renderers(&path, matches);
                } else if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                    process_rs_file(&path, matches);
                }
            }
        }
    }
}

fn process_rs_file(file_path: &PathBuf, matches: &mut HashMap<String, String>) {
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => return,
    };

    let renderer_info = match get_renderer_types(&content) {
        Some(info) => info,
        None => return,
    };

    let (renderer_type, output_type) = renderer_info;

    let full_renderer_type = build_full_renderer_type(file_path, &renderer_type);
    let full_output_type = resolve_type_paths(&content, vec![output_type])
        .unwrap()
        .get(0)
        .unwrap()
        .clone();

    matches.insert(full_renderer_type, full_output_type);
}

fn build_full_renderer_type(file_path: &PathBuf, renderer_type: &str) -> String {
    let relative_path = file_path
        .strip_prefix(std::env::current_dir().unwrap())
        .unwrap_or(file_path);
    let relative_path = relative_path.with_extension("");
    let path_str = relative_path.to_string_lossy();

    // Normalize path separators and remove "./" prefix if present
    let normalized_path = path_str
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string();

    let mut module_path = normalized_path.split('/').collect::<Vec<&str>>().join("::");

    if module_path.starts_with("src") {
        module_path = module_path.trim_start_matches("src").to_string();
        if module_path.starts_with("::") {
            module_path = module_path.trim_start_matches("::").to_string();
        }
    }

    format!("{}{}::{}", RENDERER_TYPE_PREFIX, module_path, renderer_type)
}

pub fn get_renderer_types(code: &String) -> Option<(String, String)> {
    let renderer_re = Regex::new(r"#\[result_renderer\(([^)]+)\)\]").unwrap();

    let func_re =
        Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+\w+\s*\(\s*(?:mut\s+)?\w+\s*:\s*&([^),]+)\s*")
            .unwrap();

    let code_without_comments = code
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n");

    let renderer_captures = renderer_re.captures(&code_without_comments);
    let func_captures = func_re.captures(&code_without_comments);

    match (renderer_captures, func_captures) {
        (Some(renderer_cap), Some(func_cap)) => {
            let renderer_type = renderer_cap[1].trim().to_string();
            let output_type = func_cap[1].trim().to_string();
            Some((renderer_type, output_type))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        const SITUATION: &str = "
            #[result_renderer(MyRenderer)]
            pub async fn render(data: &SomeOutput) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "MyRenderer");
        assert_eq!(output, "SomeOutput");
    }

    #[test]
    fn test2() {
        const SITUATION: &str = "
            #[result_renderer(MyRenderer)]
            pub async fn some_render(output: &SomeOutput) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "MyRenderer");
        assert_eq!(output, "SomeOutput");
    }

    #[test]
    fn test3() {
        const SITUATION: &str = "
            #[result_renderer(MyRenderer)]
            async fn some_render(output: &SomeOutput) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "MyRenderer");
        assert_eq!(output, "SomeOutput");
    }

    #[test]
    fn test4() {
        const SITUATION: &str = "
            #[result_renderer(MyRenderer)]
            async pub fn some_render(output: &SomeOutput) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "MyRenderer");
        assert_eq!(output, "SomeOutput");
    }

    #[test]
    fn test5() {
        const SITUATION: &str = "
            #[result_renderer(MyRenderer)]
            fn some_render(output: &SomeOutput2) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "MyRenderer");
        assert_eq!(output, "SomeOutput2");
    }

    #[test]
    fn test6() {
        const SITUATION: &str = "
            #[result__renderer(MyRenderer)]
            fn some_render(output: &SomeOutput2) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(
            result.is_none(),
            "Should fail to parse when annotation doesn't match"
        );
    }

    #[test]
    fn test7() {
        const SITUATION: &str = "
            #[result_renderer(MyRenderer)]
            fn some_render() -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(
            result.is_none(),
            "Should fail to parse when no function parameter"
        );
    }

    #[test]
    fn test8() {
        const SITUATION: &str = "
            #[result_renderer(MyRenderer)]
            fn some_render(output: &SomeOutput, context: &Context) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "MyRenderer");
        assert_eq!(output, "SomeOutput");
    }

    #[test]
    fn test9() {
        const SITUATION: &str = "
            #[result_renderer(MyRenderer<T>)]
            fn some_render(output: &SomeOutput<T>) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "MyRenderer<T>");
        assert_eq!(output, "SomeOutput<T>");
    }

    #[test]
    fn test10() {
        const SITUATION: &str = "
            #[result_renderer(MyRenderer<'a>)]
            fn some_render(output: &SomeOutput<'a>) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "MyRenderer<'a>");
        assert_eq!(output, "SomeOutput<'a>");
    }

    #[test]
    fn test11() {
        const SITUATION: &str = "
            #[result_renderer( MyRenderer )]
            fn some_render( output : & SomeOutput ) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "MyRenderer");
        assert_eq!(output, "SomeOutput");
    }

    #[test]
    fn test12() {
        const SITUATION: &str = "
            #[result_renderer(AnotherRenderer)]
            fn some_render(output: &DifferentOutput) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "AnotherRenderer");
        assert_eq!(output, "DifferentOutput");
    }

    #[test]
    fn test13() {
        const SITUATION: &str = "
            // #[result_renderer(WrongRenderer)]
            #[result_renderer(CorrectRenderer)]
            fn some_render(output: &CorrectOutput) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "CorrectRenderer");
        assert_eq!(output, "CorrectOutput");
    }

    #[test]
    fn test14() {
        const SITUATION: &str = "
            #[result_renderer(MultiLineRenderer)]
            fn some_render(
                output: &MultiLineOutput
            ) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "MultiLineRenderer");
        assert_eq!(output, "MultiLineOutput");
    }

    #[test]
    fn test15() {
        const SITUATION: &str = "
            #[result_renderer(MutRenderer)]
            fn some_render(mut output: &MutOutput) -> Result<JVRenderResult, CmdRenderError>
        ";

        let result = get_renderer_types(&SITUATION.to_string());
        assert!(result.is_some(), "Parse failed");
        let (renderer, output) = result.unwrap();
        assert_eq!(renderer, "MutRenderer");
        assert_eq!(output, "MutOutput");
    }
}
