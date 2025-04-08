use crate::borg::Repo;

use super::Passphrase;
use std::{fmt::Display, str::FromStr};

/// A repository specifier
///
/// This struct is used to represent a repository specifier. It can be constructed from a string
/// using the `FromStr` trait. The string can be in one of the following formats:
/// - `/path/to/repo`
/// - `path/to/repo`
/// - `~/path/to/repo`
/// - `file:///path/to/repo`
/// - `file://~/path/to/repo`
/// - `ssh://user@host:port/path/to/repo`
/// - `ssh://user@host:port/./path/to/repo`
/// - `ssh://user@host:port/~/path/to/repo`
/// - `ssh://host:port/path/to/repo`
/// - `ssh://host/path/to/repo`
///
/// Deprecated (but will be converted):
/// - `user@host:/path/to/repo`
/// - `host:/path/to/repo`
///
/// # Examples
/// ```rust
/// use borrg::Repo;
///
/// let relative: Repo = "path/to/repo".parse().unwrap();
/// assert_eq!(relative.to_string(), "path/to/repo");
///
/// let absolute: Repo = "/path/to/repo".parse().unwrap();
/// assert_eq!(absolute.to_string(), "/path/to/repo");
///
/// let in_home: Repo = "~/path/to/repo".parse().unwrap();
/// assert_eq!(in_home.to_string(), "~/path/to/repo");
///
/// let using_file: Repo = "file:///path/to/repo".parse().unwrap();
/// assert_eq!(using_file.to_string(), "/path/to/repo");
///
/// let remote_absolute: Repo = "ssh://user@host:22/path/to/repo".parse().unwrap();
/// assert_eq!(remote_absolute.to_string(), "ssh://user@host:22/path/to/repo");
///
/// let remote_relative: Repo = "ssh://user@host:22/./path/to/repo".parse().unwrap();
/// assert_eq!(remote_relative.to_string(), "ssh://user@host:22/./path/to/repo");
///
/// let remote_in_home: Repo = "ssh://user@host:22/~/path/to/repo".parse().unwrap();
/// assert_eq!(remote_in_home.to_string(), "ssh://user@host:22/~/path/to/repo");
///
/// #[cfg(feature = "deprecated")]
/// let old: Repo = "user@host:/path/to/repo".parse().unwrap();
/// #[cfg(feature = "deprecated")]
/// assert_eq!(old.to_string(), "ssh://user@host//path/to/repo");
/// ```
#[derive(Debug, Clone, Eq)]
pub struct RepoConfig {
    pub url: Repo<'static>,
    pub(crate) passphrase: Option<Passphrase>,
}

impl FromStr for RepoConfig {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = s.parse()?;
        Ok(Self {
            url,
            passphrase: None,
        })
    }
}

impl Display for RepoConfig {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.url.fmt(f)
    }
}

impl PartialEq for RepoConfig {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}
