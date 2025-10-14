pub fn current_locales() -> String {
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
