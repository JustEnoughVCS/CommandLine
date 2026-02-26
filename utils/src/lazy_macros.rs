/// A macro for creating a `Vec<String>` from string literals.
///
/// # Examples
/// ```
/// let v = string_vec!["hello", "world"];
/// assert_eq!(v, vec!["hello".to_string(), "world".to_string()]);
/// ```
#[macro_export]
macro_rules! string_vec {
    ($($elem:expr),* $(,)?) => {
        vec![$($elem.to_string()),*]
    };
}
