use core::num::NonZeroU16;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

pub trait Repository {
    #[inline]
    fn path(&self) -> Option<&Path> {
        None
    }

    #[inline]
    fn user(&self) -> Option<&str> {
        None
    }

    #[inline]
    fn host(&self) -> Option<&str> {
        None
    }

    #[inline]
    fn port(&self) -> Option<NonZeroU16> {
        None
    }

    fn protocol(&self) -> &str;

    fn repo_url(&self) -> Cow<str>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ssh<'a> {
    pub user: Option<Cow<'a, str>>,
    pub host: Cow<'a, str>,
    pub port: Option<NonZeroU16>,
    pub path: Cow<'a, Path>,
}

impl Repository for Ssh<'_> {
    #[inline]
    fn host(&self) -> Option<&str> {
        Some(self.host.as_ref())
    }

    #[inline]
    fn path(&self) -> Option<&Path> {
        Some(self.path.as_ref())
    }

    #[inline]
    fn port(&self) -> Option<NonZeroU16> {
        self.port
    }

    #[inline]
    fn user(&self) -> Option<&str> {
        self.user.as_deref()
    }

    #[inline]
    fn protocol(&self) -> &str {
        "ssh"
    }

    #[inline]
    fn repo_url(&self) -> Cow<str> {
        let mut url = String::new();
        url.push_str("ssh://");
        if let Some(user) = &self.user {
            url.push_str(user.as_ref());
            url.push('@');
        }
        url.push_str(self.host.as_ref());
        if let Some(port) = self.port {
            url.push(':');
            url.push_str(&port.to_string());
        }
        url.push_str(":");
        url.push_str(self.host.as_ref());
        Cow::Owned(url)
    }
}

pub struct Sftp<'a> {
    pub user: Option<Cow<'a, str>>,
    pub host: Cow<'a, str>,
    pub port: Option<NonZeroU16>,
    pub path: Cow<'a, Path>,
}

impl Repository for Sftp<'_> {
    #[inline]
    fn host(&self) -> Option<&str> {
        Some(self.host.as_ref())
    }

    #[inline]
    fn path(&self) -> Option<&Path> {
        Some(self.path.as_ref())
    }

    #[inline]
    fn port(&self) -> Option<NonZeroU16> {
        self.port
    }

    #[inline]
    fn user(&self) -> Option<&str> {
        self.user.as_deref()
    }

    #[inline]
    fn protocol(&self) -> &str {
        "sftp"
    }

    #[inline]
    fn repo_url(&self) -> Cow<str> {
        let mut url = String::new();
        url.push_str("sftp://");
        if let Some(user) = &self.user {
            url.push_str(user.as_ref());
            url.push('@');
        }
        url.push_str(self.host.as_ref());
        if let Some(port) = self.port {
            url.push(':');
            url.push_str(&port.to_string());
        }
        url.push_str(":");
        url.push_str(self.host.as_ref());
        Cow::Owned(url)
    }
}

pub struct RClone<'a> {
    pub remote: Cow<'a, str>,
    pub path: Cow<'a, Path>,
}

impl Repository for RClone<'_> {
    #[inline]
    fn path(&self) -> Option<&Path> {
        Some(self.path.as_ref())
    }

    #[inline]
    fn protocol(&self) -> &str {
        "rclone"
    }

    #[inline]
    fn repo_url(&self) -> Cow<str> {
        let mut url = String::new();
        url.push_str("rclone:");
        url.push_str(self.remote.as_ref());
        url.push(':');
        url.push_str(self.path.as_ref().to_string_lossy().as_ref());
        Cow::Owned(url)
    }
}

impl Repository for Cow<'_, Path> {
    #[inline]
    fn path(&self) -> Option<&Path> {
        Some(self.as_ref())
    }

    #[inline]
    fn protocol(&self) -> &str {
        "file"
    }

    #[inline]
    fn repo_url(&self) -> Cow<str> {
        self.to_string_lossy()
    }
}

impl Repository for PathBuf {
    #[inline]
    fn path(&self) -> Option<&Path> {
        Some(self.as_path())
    }

    #[inline]
    fn protocol(&self) -> &str {
        "file"
    }

    #[inline]
    fn repo_url(&self) -> Cow<str> {
        self.to_string_lossy()
    }
}

impl Repository for String {
    #[inline]
    fn protocol(&self) -> &str {
        match self.as_str() {
            _ if self.starts_with("ssh://") => "ssh",
            _ if self.starts_with("sftp://") => "sftp",
            _ if self.starts_with("rclone:") => "rclone",
            _ => "file",
        }
    }

    #[inline]
    fn repo_url(&self) -> Cow<str> {
        Cow::Borrowed(self.as_str())
    }
}

impl Repository for std::sync::Arc<dyn Repository + '_> {
    #[inline]
    fn path(&self) -> Option<&Path> {
        self.as_ref().path()
    }

    #[inline]
    fn user(&self) -> Option<&str> {
        self.as_ref().user()
    }

    #[inline]
    fn host(&self) -> Option<&str> {
        self.as_ref().host()
    }

    #[inline]
    fn port(&self) -> Option<NonZeroU16> {
        self.as_ref().port()
    }

    #[inline]
    fn protocol(&self) -> &str {
        self.as_ref().protocol()
    }

    #[inline]
    fn repo_url(&self) -> Cow<str> {
        self.as_ref().repo_url()
    }
}

impl core::fmt::Debug for dyn Repository + '_ {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Repository")
            .field("repo_url", &self.repo_url())
            .field("protocol", &self.protocol())
            .field("path", &self.path())
            .field("user", &self.user())
            .field("host", &self.host())
            .field("port", &self.port())
            .finish()
    }
}

#[inline]
pub fn parse(value: &toml::Value) -> Result<std::sync::Arc<dyn Repository>, &'static str> {
    use std::sync::Arc;
    if let Some(repo) = value.as_str() {
        return Ok(Arc::new(repo.to_string()));
    }

    let Some(table) = value.as_table() else {
        return Err("Invalid repository format");
    };

    let protocol = table.get("protocol").and_then(|v| v.as_str());
    let path = table.get("path").and_then(|v| v.as_str());
    let user = table.get("user").and_then(|v| v.as_str());
    let host = table.get("host").and_then(|v| v.as_str());
    let port = table
        .get("port")
        .and_then(|v| v.as_integer())
        .and_then(|p| NonZeroU16::new(p as u16));
    let remote = table.get("remote").and_then(|v| v.as_str());

    return match protocol {
        Some("file") | None => {
            let Some(path) = path else {
                return Err("Missing path in repository");
            };
            Ok(Arc::new(PathBuf::from(path)))
        }
        Some("ssh") => {
            let Some(host) = host else {
                return Err("Missing host in repository");
            };
            let Some(path) = path else {
                return Err("Missing path in repository");
            };

            Ok(Arc::new(Ssh {
                host: host.to_owned().into(),
                path: PathBuf::from(path).into(),
                port,
                user: user.map(|u| u.to_owned().into()),
            }))
        }
        Some("sftp") => {
            let Some(host) = host else {
                return Err("Missing host in repository");
            };
            let Some(path) = path else {
                return Err("Missing path in repository");
            };

            Ok(Arc::new(Sftp {
                host: host.to_owned().into(),
                path: PathBuf::from(path).into(),
                port,
                user: user.map(|u| u.to_owned().into()),
            }))
        }
        Some("rclone") => {
            let Some(remote) = remote else {
                return Err("Missing remote in repository");
            };
            let Some(path) = path else {
                return Err("Missing path in repository");
            };

            Ok(Arc::new(RClone {
                remote: remote.to_owned().into(),
                path: PathBuf::from(path).into(),
            }))
        }
        _ => Err("Invalid protocol"),
    };
}
