use super::Passphrase;
use std::{fmt::Display, path::PathBuf, str::FromStr};

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
/// let old: Repo = "user@host:/path/to/repo".parse().unwrap();
/// assert_eq!(old.to_string(), "ssh://user@host/path/to/repo");
/// ```
#[derive(Debug, Clone, Eq)]
pub struct Repo {
    remote: Option<Remote>,
    pub(crate) path: PathBuf,
    pub(crate) passphrase: Option<Passphrase>,
}

impl FromStr for Repo {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(path) = s.strip_prefix("file://") {
            return Ok(Self {
                remote: None,
                path: path.into(),
                passphrase: None,
            });
        }

        if let Some(repo) = s.strip_prefix("ssh://") {
            let (remote, path) = repo
                .split_once('/')
                .ok_or("Invalid repository specifier (No \"/\" after \"ssh://\")")?;
            let remote = remote.parse()?;
            if !path.starts_with('.') && !path.starts_with('~') {
                return Ok(Self {
                    remote: Some(remote),
                    path: PathBuf::from("/").join(path),
                    passphrase: None,
                });
            }
            return Ok(Self {
                remote: Some(remote),
                path: path.into(),
                passphrase: None,
            });
        }

        if let Some((remote, path)) = s.split_once(':') {
            log::warn!(
                "Repository specifier without protocol (\"ssh://\") is deprecated and will be removed in borg 2.\n\
                Please use \"ssh://{remote}/{path}\" instead.\n\
                Note: borrg will still support the old format by converting it internally."
            );
            let remote = remote.parse()?;
            return Ok(Self {
                remote: Some(remote),
                path: path.into(),
                passphrase: None,
            });
        }

        Ok(Self {
            remote: None,
            path: s.into(),
            passphrase: None,
        })
    }
}

// impl From<String> for Repo {
//     fn from(s: String) -> Self {
//         todo!()
//     }
// }

impl Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(remote) = &self.remote {
            write!(f, "ssh://{remote}")?;
            if self.path.is_relative() {
                write!(f, "/")?;
            }
        }

        write!(f, "{}", self.path.display())
    }
}

impl PartialEq for Repo {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.remote == other.remote
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Remote {
    user: Option<String>,
    host: String,
    port: Option<u16>,
}

impl FromStr for Remote {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut user = None;
        let mut port = None;

        let mut rest = s;
        if let Some((u, h)) = rest.split_once('@') {
            user.replace(u.to_string());
            rest = h;
        }
        if let Some((h, p)) = rest.split_once(':') {
            port.replace(
                p.parse()
                    .map_err(|_| "Invalid remote: Failed to parse port")?,
            );
            rest = h;
        }

        Ok(Self {
            user,
            host: rest.to_string(),
            port,
        })
    }
}

impl Display for Remote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(user) = &self.user {
            write!(f, "{user}@")?;
        }
        write!(f, "{}", self.host)?;
        if let Some(port) = &self.port {
            write!(f, ":{port}")?;
        }
        Ok(())
    }
}
