use std::{borrow::Cow, path::Path};

use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoUrl<'a> {
    Local(Cow<'a, Path>),
    Ssh {
        user: Option<SmolStr>,
        host: SmolStr,
        path: Cow<'a, Path>,
    },
    Sftp {
        user: Option<SmolStr>,
        host: SmolStr,
        path: Cow<'a, Path>,
    },
    RClone {
        remote: SmolStr,
        path: Cow<'a, Path>,
    },
}

impl RepoUrl<'_> {
    #[inline]
    #[must_use]
    pub const fn is_local(&self) -> bool {
        matches!(self, RepoUrl::Local(..))
    }

    #[inline]
    #[must_use]
    pub const fn protocol(&self) -> &'static str {
        match *self {
            RepoUrl::Local(..) => "file",
            RepoUrl::Ssh { .. } => "ssh",
            RepoUrl::Sftp { .. } => "sftp",
            RepoUrl::RClone { .. } => "rclone",
        }
    }

    #[inline]
    #[must_use]
    pub fn path(&self) -> &Path {
        match self {
            RepoUrl::Local(path)
            | RepoUrl::Ssh { path, .. }
            | RepoUrl::Sftp { path, .. }
            | RepoUrl::RClone { path, .. } => path,
        }
    }

    #[inline]
    #[must_use]
    pub fn host(&self) -> Option<&str> {
        match self {
            RepoUrl::Ssh { host, .. } | RepoUrl::Sftp { host, .. } => Some(host.as_ref()),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_smol_str(&self) -> SmolStr {
        let ssh = ['s', 's', 'h', ':', '/', '/'].into_iter();
        let sftp = ['s', 'f', 't', 'p', ':', '/', '/'].into_iter();
        let rclone = ['r', 'c', 'l', 'o', 'n', 'e', ':'].into_iter();
        let at = ['@'];
        let slash = ['/'];
        let colon = [':'];

        match self {
            Self::Local(path) => path.to_string_lossy().into(),
            Self::Ssh {
                user: None,
                host,
                path,
            } => ssh
                .chain(host.chars())
                .chain(slash)
                .chain(path.to_string_lossy().chars())
                .collect(),
            Self::Ssh {
                user: Some(user),
                host,
                path,
            } => ssh
                .chain(user.chars())
                .chain(at)
                .chain(host.chars())
                .chain(slash)
                .chain(path.to_string_lossy().chars())
                .collect(),
            Self::Sftp {
                user: None,
                host,
                path,
            } => sftp
                .chain(host.chars())
                .chain(slash)
                .chain(path.to_string_lossy().chars())
                .collect(),
            Self::Sftp {
                user: Some(user),
                host,
                path,
            } => sftp
                .chain(user.chars())
                .chain(at)
                .chain(host.chars())
                .chain(slash)
                .chain(path.to_string_lossy().chars())
                .collect(),
            Self::RClone { remote, path } => rclone
                .chain(remote.chars())
                .chain(colon)
                .chain(path.to_string_lossy().chars())
                .collect(),
        }
    }
}

impl core::fmt::Display for RepoUrl<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.as_smol_str().fmt(f)
    }
}

impl std::str::FromStr for RepoUrl<'static> {
    type Err = &'static str;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let home = Cow::Borrowed(Path::new("~"));
        let root = Cow::Borrowed(Path::new("/"));

        let url = match s {
            "~" => Self::Local(home),
            "/" => Self::Local(root),
            s if s.starts_with("file://") => {
                let path = Cow::Owned(s[7..].into());
                Self::Local(path)
            }
            s if s.starts_with("ssh://") => {
                let mut rest = &s[6..];
                let user = match rest.split_once('@') {
                    None => None,
                    Some((user, r)) => {
                        rest = r;
                        Some(user.into())
                    }
                };
                match rest.split_once('/') {
                    None => return Err("missing /"),
                    Some((host, "/")) => Self::Ssh {
                        user,
                        host: host.into(),
                        path: root,
                    },
                    Some((host, path)) => Self::Ssh {
                        user,
                        host: host.into(),
                        path: Cow::Owned(path.into()),
                    },
                }
            }
            s if s.starts_with("sftp://") => {
                let mut rest = &s[7..];
                let user = match rest.split_once('@') {
                    None => None,
                    Some((user, r)) => {
                        rest = r;
                        Some(user.into())
                    }
                };
                match rest.split_once('/') {
                    None => return Err("missing /"),
                    Some((host, "/")) => Self::Sftp {
                        user,
                        host: host.into(),
                        path: root,
                    },
                    Some((host, path)) => Self::Sftp {
                        user,
                        host: host.into(),
                        path: Cow::Owned(path.into()),
                    },
                }
            }
            s if s.starts_with("rclone:") => match s[7..].split_once(':') {
                None => return Err("invalid rclone url"),
                Some((remote, path)) => Self::RClone {
                    remote: remote.into(),
                    path: Cow::Owned(path.into()),
                },
            },
            #[cfg(feature = "deprecated")]
            path if path.contains(':') => {
                let (host, path) = path.split_once(':').unwrap();
                match host.split_once('@') {
                    None => Self::Ssh {
                        user: None,
                        host: host.into(),
                        path: Cow::Owned(path.into()),
                    },
                    Some((user, host)) => Self::Ssh {
                        user: Some(user.into()),
                        host: host.into(),
                        path: Cow::Owned(path.into()),
                    },
                }
            }
            path => Self::Local(Cow::Owned(path.into())),
        };

        Ok(url)
    }
}

impl TryFrom<&toml::Value> for RepoUrl<'static> {
    type Error = &'static str;

    #[inline]
    fn try_from(value: &toml::Value) -> Result<Self, Self::Error> {
        use toml::Value;

        let table = match value {
            Value::String(s) => return s.parse(),
            Value::Table(tab) => tab,
            _ => return Err("invalid repository"),
        };

        let Some(path) = table.get("path").and_then(Value::as_str) else {
            return Err("missing path");
        };

        let host = table.get("host").and_then(Value::as_str);

        let user = table.get("user").and_then(Value::as_str);

        let remote = table.get("remote").and_then(Value::as_str);

        let url = match table.get("protocol") {
            None if remote.is_some() => Self::RClone {
                remote: remote.unwrap().into(),
                path: Cow::Owned(path.into()),
            },
            None if host.is_some() => Self::Ssh {
                user: user.map(SmolStr::new),
                host: host.unwrap().into(),
                path: Cow::Owned(path.into()),
            },
            Some(v) if v.as_str() == Some("ssh") => Self::Ssh {
                user: user.map(SmolStr::new),
                host: host.ok_or("missing host")?.into(),
                path: Cow::Owned(path.into()),
            },
            Some(v) if v.as_str() == Some("sftp") => Self::Sftp {
                user: user.map(SmolStr::new),
                host: host.ok_or("missing host")?.into(),
                path: Cow::Owned(path.into()),
            },
            Some(v) if v.as_str() == Some("rclone") => Self::RClone {
                remote: remote.ok_or("missing remote")?.into(),
                path: Cow::Owned(path.into()),
            },
            Some(v) if v.as_str() == Some("file") => Self::Local(Cow::Owned(path.into())),
            Some(_) => return Err("invalid protocol"),
            None => Self::Local(Cow::Owned(path.into())),
        };

        Ok(url)
    }
}
