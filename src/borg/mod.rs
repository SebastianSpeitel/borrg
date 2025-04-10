use std::os::fd::RawFd;

use smol_str::SmolStr;

pub mod archive;
pub mod compression;
pub mod repo;

pub use archive::Archive;
pub use compression::Compression;
pub use repo::Repo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemotePath(pub SmolStr);

impl Default for RemotePath {
    #[inline]
    fn default() -> Self {
        Self(SmolStr::new_static("borg"))
    }
}

impl TryFrom<&toml::Value> for RemotePath {
    type Error = &'static str;

    #[inline]
    fn try_from(value: &toml::Value) -> Result<Self, Self::Error> {
        use toml::Value;
        match *value {
            Value::String(ref s) => Ok(Self(s.into())),
            _ => Err("Invalid remote path"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Passphrase {
    #[default]
    None,
    Phrase(SmolStr),
    Command(SmolStr),
    Fd(RawFd),
}

impl TryFrom<&toml::Value> for Passphrase {
    type Error = &'static str;

    #[inline]
    fn try_from(value: &toml::Value) -> Result<Self, Self::Error> {
        use toml::Value;
        let pass = match *value {
            Value::Boolean(false) => Self::None,
            Value::Integer(i) => Self::Fd(i.try_into().map_err(|_| "Invalid fd")?),
            Value::String(ref p) => Self::Phrase(p.into()),
            Value::Table(ref t) => {
                if let Some(cmd) = t.get("command").and_then(Value::as_str) {
                    Self::Command(cmd.into())
                } else {
                    return Err("Invalid passphrase");
                }
            }
            _ => return Err("Invalid passphrase"),
        };

        Ok(pass)
    }
}
