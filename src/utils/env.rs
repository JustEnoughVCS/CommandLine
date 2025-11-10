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
