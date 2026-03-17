use regex::Regex;

pub fn resolve_type_paths(code: &String, type_names: Vec<String>) -> Option<Vec<String>> {
    let mut type_mappings = std::collections::HashMap::new();

    // Extract all use statements
    let use_re = Regex::new(r"use\s+([^;]*(?:\{[^}]*\}[^;]*)*);").ok()?;
    let mut use_statements = Vec::new();
    for cap in use_re.captures_iter(&code) {
        use_statements.push(cap[1].to_string());
    }

    // Process each use statement to build type mappings
    for stmt in &use_statements {
        let stmt = stmt.trim();

        if stmt.contains("::{") {
            if let Some(pos) = stmt.find("::{") {
                let base_path = &stmt[..pos];
                let content = &stmt[pos + 3..stmt.len() - 1];
                process_nested_use(base_path, content, &mut type_mappings);
            }
        } else {
            // Process non-nested use statements
            if let Some(pos) = stmt.rfind("::") {
                let type_name = &stmt[pos + 2..];
                type_mappings.insert(type_name.to_string(), stmt.to_string());
            } else {
                type_mappings.insert(stmt.to_string(), stmt.to_string());
            }
        }
    }

    // Resolve type names to full paths
    let mut result = Vec::new();
    for type_name in type_names {
        if let Some(full_path) = type_mappings.get(&type_name) {
            result.push(full_path.clone());
        }
    }

    Some(result)
}

fn process_nested_use(
    base_path: &str,
    content: &str,
    mappings: &mut std::collections::HashMap<String, String>,
) {
    let mut items = Vec::new();
    let mut current_item = String::new();
    let mut brace_depth = 0;

    // Split nested content
    for c in content.chars() {
        match c {
            '{' => {
                brace_depth += 1;
                current_item.push(c);
            }
            '}' => {
                brace_depth -= 1;
                current_item.push(c);
            }
            ',' => {
                if brace_depth == 0 {
                    items.push(current_item.trim().to_string());
                    current_item.clear();
                } else {
                    current_item.push(c);
                }
            }
            _ => {
                current_item.push(c);
            }
        }
    }

    if !current_item.trim().is_empty() {
        items.push(current_item.trim().to_string());
    }

    // Process each item
    for item in items {
        if item.is_empty() {
            continue;
        }

        if item.contains("::{") {
            if let Some(pos) = item.find("::{") {
                let sub_path = &item[..pos];
                let sub_content = &item[pos + 3..item.len() - 1];
                let new_base = if base_path.is_empty() {
                    sub_path.to_string()
                } else {
                    format!("{}::{}", base_path, sub_path)
                };
                process_nested_use(&new_base, sub_content, mappings);
            }
        } else {
            let full_path = if base_path.is_empty() {
                item.to_string()
            } else {
                format!("{}::{}", base_path, item)
            };
            if let Some(pos) = item.rfind("::") {
                let type_name = &item[pos + 2..];
                mappings.insert(type_name.to_string(), full_path);
            } else {
                mappings.insert(item.to_string(), full_path);
            }
        }
    }
}
