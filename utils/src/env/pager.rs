/// Gets the default pager based on environment variables.
///
/// The function checks the JV_PAGER environment variable
/// and returns its value if it is set. If the variable is not set,
/// it returns "less" as the default pager.
///
/// # Returns
/// A String containing the default pager
pub async fn get_default_pager() -> String {
    if let Ok(pager) = std::env::var("JV_PAGER") {
        return pager;
    }

    "less".to_string()
}
