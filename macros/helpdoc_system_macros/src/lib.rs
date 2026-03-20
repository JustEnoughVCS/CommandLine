use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use std::path::Path;

#[proc_macro]
pub fn generate_helpdoc_mapping(_input: TokenStream) -> TokenStream {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    let repo_root = Path::new(&manifest_dir);
    let helpdoc_dir = repo_root.join("resources").join("helpdoc");

    if !helpdoc_dir.exists() {
        return quote! {
            fn get_doc(_doc_name: &str, _lang: &str) -> &'static str {
                ""
            }
        }
        .into();
    }

    let mut doc_entries = Vec::new();

    scan_directory(&helpdoc_dir, &mut doc_entries, &helpdoc_dir);

    let match_arms = generate_match_arms(&doc_entries);

    let expanded = quote! {
        fn get_doc(doc_name: &str, lang: &str) -> &'static str {
            let key = format!("{}.{}", doc_name, lang);
            match key.as_str() {
                #(#match_arms)*
                _ => "",
            }
        }
    };

    expanded.into()
}

fn scan_directory(dir: &Path, entries: &mut Vec<(String, String)>, base_dir: &Path) {
    if let Ok(entries_iter) = fs::read_dir(dir) {
        for entry in entries_iter.filter_map(Result::ok) {
            let path = entry.path();

            if path.is_dir() {
                scan_directory(&path, entries, base_dir);
            } else if let Some(extension) = path.extension()
                && extension == "md"
                    && let Ok(relative_path) = path.strip_prefix(base_dir)
                        && let Some(file_stem) = path.file_stem() {
                            let file_stem_str = file_stem.to_string_lossy();

                            if let Some(dot_pos) = file_stem_str.rfind('.') {
                                let doc_name = &file_stem_str[..dot_pos];
                                let lang = &file_stem_str[dot_pos + 1..];

                                let parent = relative_path.parent();
                                let full_doc_name = if let Some(parent) = parent {
                                    if parent.to_string_lossy().is_empty() {
                                        doc_name.to_string()
                                    } else {
                                        format!("{}/{}", parent.to_string_lossy(), doc_name)
                                    }
                                } else {
                                    doc_name.to_string()
                                };

                                let full_doc_name = just_fmt::fmt_path::fmt_path(full_doc_name)
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string();
                                entries.push((full_doc_name, lang.to_string()));
                            }
                        }
        }
    }
}

fn generate_match_arms(entries: &[(String, String)]) -> Vec<proc_macro2::TokenStream> {
    let mut arms = Vec::new();

    for (doc_name, lang) in entries {
        let key = format!("{}.{}", doc_name, lang);
        let file_path = format!("resources/helpdoc/{}.{}.md", doc_name, lang);

        let arm = quote! {
            #key => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #file_path)),
        };

        arms.push(arm);
    }

    arms
}

#[proc_macro]
pub fn generate_helpdoc_list(_input: TokenStream) -> TokenStream {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    let repo_root = Path::new(&manifest_dir);
    let helpdoc_dir = repo_root.join("resources").join("helpdoc");

    if !helpdoc_dir.exists() {
        return quote! {
            fn get_docs_list() -> Vec<&'static str> {
                Vec::new()
            }
        }
        .into();
    }

    let mut doc_entries = Vec::new();
    scan_directory(&helpdoc_dir, &mut doc_entries, &helpdoc_dir);

    let mut unique_docs = std::collections::HashSet::new();
    for (doc_name, _) in &doc_entries {
        unique_docs.insert(doc_name.clone());
    }

    let mut doc_list = Vec::new();
    for doc_name in unique_docs {
        doc_list.push(quote! {
            #doc_name
        });
    }

    let expanded = quote! {
        fn get_docs_list() -> Vec<&'static str> {
            vec![
                #(#doc_list),*
            ]
        }
    };

    expanded.into()
}

#[proc_macro]
pub fn generate_helpdoc_test(_input: TokenStream) -> TokenStream {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    let repo_root = Path::new(&manifest_dir);
    let helpdoc_dir = repo_root.join("resources").join("helpdoc");

    if !helpdoc_dir.exists() {
        return quote! {
            #[cfg(test)]
            mod helpdoc_tests {
                #[test]
                fn test_no_docs() {
                }
            }
        }
        .into();
    }

    let mut doc_entries = Vec::new();
    scan_directory(&helpdoc_dir, &mut doc_entries, &helpdoc_dir);

    let mut test_cases = Vec::new();

    for (doc_name, lang) in &doc_entries {
        let test_name_str = format!(
            "test_doc_{}_{}",
            doc_name
                .replace(['/', '.', '-'], "_"),
            lang.replace('-', "_")
        );
        let test_name = syn::Ident::new(&test_name_str, proc_macro2::Span::call_site());
        let test_case = quote! {
            #[test]
            fn #test_name() {
                let doc = super::get_doc(#doc_name, #lang);
                assert!(!doc.is_empty(), "Document {}.{} should not be empty", #doc_name, #lang);
            }
        };

        test_cases.push(test_case);
    }

    let expanded = quote! {
        #[cfg(test)]
        mod helpdoc_tests {
            #(#test_cases)*
        }
    };

    expanded.into()
}
