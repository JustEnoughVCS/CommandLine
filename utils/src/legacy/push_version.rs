pub fn push_version(current_version: impl Into<String>) -> Option<String> {
    let version_str = current_version.into();
    let parts: Vec<&str> = version_str.split('.').collect();

    if parts.len() != 3 {
        return None;
    }

    let major: Result<u32, _> = parts[0].parse();
    let minor: Result<u32, _> = parts[1].parse();
    let patch: Result<u32, _> = parts[2].parse();

    if let (Ok(mut major), Ok(mut minor), Ok(mut patch)) = (major, minor, patch) {
        patch += 1;

        if patch > 99 {
            patch = 0;
            minor += 1;

            if minor > 99 {
                minor = 0;
                major += 1;
            }
        }

        Some(format!("{}.{}.{}", major, minor, patch))
    } else {
        None
    }
}
