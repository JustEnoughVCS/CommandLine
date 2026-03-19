/// Gets the default help documentation setting based on environment variables.
///
/// The function checks the JV_HELPDOC_VIEWER environment variable.
/// If the variable is set to "1", it returns true (enabled).
/// If the variable is set to any other value, it returns false (disabled).
/// If the variable is not set, it returns true (enabled by default).
///
/// # Returns
/// A boolean indicating whether help documentation is enabled
pub fn get_helpdoc_enabled() -> bool {
    match std::env::var("JV_HELPDOC_VIEWER") {
        Ok(value) => value == "1",
        Err(_) => true,
    }
}
