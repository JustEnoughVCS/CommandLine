// /// Returns the current locale string based on environment variables.
///
/// The function checks for locale settings in the following order:
/// 1. JV_LANG environment variable
/// 2. APP_LANG environment variable
/// 3. LANG environment variable (extracts base language before dot and replaces underscores with hyphens)
/// 4. Defaults to "en" if no locale environment variables are found
///
/// # Returns
/// A String containing the detected locale code
pub fn current_locales() -> String {
    if let Ok(lang) = std::env::var("JV_LANG") {
        return lang;
    }

    if let Ok(lang) = std::env::var("APP_LANG") {
        return lang;
    }

    if let Ok(lang) = std::env::var("LANG") {
        if let Some(base_lang) = lang.split('.').next() {
            return base_lang.replace('_', "-");
        }
        return lang;
    }

    "en".to_string()
}

/// Checks if auto update is enabled based on environment variables.
///
/// The function checks the JV_AUTO_UPDATE environment variable and compares
/// its value (after trimming and converting to lowercase) against known
/// positive and negative values.
///
/// # Returns
/// `true` if the value matches "yes", "y", or "true"
/// `false` if the value matches "no", "n", or "false", or if the variable is not set
pub fn enable_auto_update() -> bool {
    if let Ok(auto_update) = std::env::var("JV_AUTO_UPDATE") {
        let normalized = auto_update.trim().to_lowercase();
        match normalized.as_str() {
            "yes" | "y" | "true" => return true,
            "no" | "n" | "false" => return false,
            _ => {}
        }
    }
    false
}

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
