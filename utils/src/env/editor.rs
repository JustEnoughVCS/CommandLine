/// Gets the default text editor based on environment variables.
///
/// The function checks the JV_TEXT_EDITOR and EDITOR environment variables
/// and returns their values if they are set. If neither variable is set,
/// it returns "jvii" as the default editor.
///
/// # Returns
/// A String containing the default text editor
pub async fn get_default_editor() -> String {
    if let Ok(editor) = std::env::var("JV_TEXT_EDITOR") {
        return editor;
    }

    if let Ok(editor) = std::env::var("EDITOR") {
        return editor;
    }

    "jvii".to_string()
}
