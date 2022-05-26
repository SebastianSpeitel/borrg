use std::path::PathBuf;

#[inline]
pub fn resolve_path(path: &PathBuf) -> PathBuf {
    if path == &PathBuf::from("~") {
        return dirs::home_dir().unwrap();
    }

    match path.strip_prefix("~/") {
        Ok(path) => dirs::home_dir().unwrap().join(path),
        Err(_) => path.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path() {
        let should_resolve = PathBuf::from("~/test");
        assert_ne!(should_resolve, resolve_path(&should_resolve));

        let should_not_resolve = PathBuf::from("/test");
        assert_eq!(should_not_resolve, resolve_path(&should_not_resolve));

        let should_not_resolve = PathBuf::from("~test");
        assert_eq!(should_not_resolve, resolve_path(&should_not_resolve));

        let home_only = PathBuf::from("~");
        assert_ne!(home_only, resolve_path(&home_only));
    }
}
