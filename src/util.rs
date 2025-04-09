use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    sync::LazyLock,
};

static HOME: LazyLock<Option<PathBuf>> = LazyLock::new(dirs::home_dir);

pub trait PathResolveExt: AsRef<Path> {
    #[inline]
    fn resolve(&self) -> Cow<Path> {
        let path = self.as_ref();
        if path == Path::new("~") {
            return Cow::Borrowed(HOME.as_ref().unwrap());
        }

        if let Ok(path) = path.strip_prefix("~/") {
            return Cow::Owned(HOME.as_ref().unwrap().join(path));
        }

        Cow::Borrowed(path)
    }
}

impl<T: AsRef<Path>> PathResolveExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path() {
        let should_resolve = PathBuf::from("~/test");
        assert_ne!(should_resolve, should_resolve.resolve());

        let should_not_resolve = PathBuf::from("/test");
        assert_eq!(should_not_resolve, should_not_resolve.resolve());

        let should_not_resolve = PathBuf::from("~test");
        assert_eq!(should_not_resolve, should_not_resolve.resolve());

        let home_only = PathBuf::from("~");
        assert_ne!(home_only, home_only.resolve());
    }
}
