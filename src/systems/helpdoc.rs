pub mod helpdoc_viewer;

helpdoc_system_macros::generate_helpdoc_mapping!();
helpdoc_system_macros::generate_helpdoc_list!();
helpdoc_system_macros::generate_helpdoc_test!();

pub fn get_helpdoc<'a>(doc_name: &'a str, lang: &'a str) -> &'a str {
    let doc = get_doc(doc_name, lang);
    if doc.is_empty() && lang != "en" {
        get_doc(doc_name, "en")
    } else {
        doc
    }
}

pub fn get_helpdoc_list<'a>() -> Vec<&'a str> {
    get_docs_list()
}
