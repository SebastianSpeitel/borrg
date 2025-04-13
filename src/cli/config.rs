use std::collections::BTreeMap;
use std::path::PathBuf;

use smol_str::SmolStr;
use thiserror::Error;

use crate::borg::{Archive, Options, Repo};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid type: expected {expected:?}, found {found:?}")]
    TypeError {
        expected: Option<&'static str>,
        found: Option<&'static str>,
    },
    #[error("Invalid value")]
    ValueError,
    #[error("Missing key \"{0}\"")]
    MissingKey(&'static str),
    #[error("Keys {0} and {1} are exclusive")]
    ExclusiveKeys(&'static str, &'static str),
    #[error("Missing template \"{0}\"")]
    MissingTemplate(String),
    #[error("Error at key \"{key}\": {err}")]
    Keyed { key: String, err: Box<ConfigError> },
    #[error(transparent)]
    IOError(std::io::Error),
    #[error(transparent)]
    ParseError(toml::de::Error),
    #[error("Error: {0}")]
    Other(&'static str),
}

impl From<&'static str> for ConfigError {
    #[inline]
    fn from(value: &'static str) -> Self {
        Self::Other(value)
    }
}

impl std::process::Termination for ConfigError {
    fn report(self) -> std::process::ExitCode {
        eprintln!("{self}");
        std::process::ExitCode::FAILURE
    }
}

#[derive(Debug, Clone, Default)]
pub struct PartialBackup {
    pub template: Option<SmolStr>,
    pub repo: Option<Repo<'static>>,
    pub archive_name: Option<SmolStr>,
    pub options: Options,
}

impl PartialBackup {
    #[inline]
    pub fn resolve(self, templates: &BTreeMap<SmolStr, Self>) -> Result<Archive, ConfigError> {
        let template = self.template.ok_or("Root template can't be resolved")?;

        let template = match templates.get(&template) {
            Some(t) => t,
            None if template == "default" => &Self::default(),
            None => return Err(ConfigError::MissingTemplate(template.to_string())),
        };

        let repo = match (self.repo, &template.repo) {
            (Some(r), ..) => r,
            (None, Some(r)) => r.clone(),
            _ => return Err(ConfigError::MissingKey("repository")),
        };

        let name = match (self.archive_name, &template.archive_name) {
            (Some(n), ..) => n,
            (None, Some(n)) => n.clone(),
            _ => {
                if cfg!(feature = "borg1-compat") {
                    SmolStr::new_static("{hostname}-{now:%Y-%m-%dT%H:%M:%S.%f}")
                } else {
                    SmolStr::new_static("{hostname}")
                }
            }
        };

        let options = template.options.clone() + self.options;

        Ok(Archive {
            repo,
            name,
            options,
        })
    }
}

impl TryFrom<&toml::Value> for PartialBackup {
    type Error = &'static str;

    #[inline]
    fn try_from(value: &toml::Value) -> Result<Self, Self::Error> {
        use toml::Value;
        let tab = match value {
            Value::Table(t) => t,
            _ => return Err("Invalid backup"),
        };

        let repo = match tab.get("repository") {
            None => None,
            Some(v) => Some(Repo::try_from(v)?),
        };

        let template = match tab.get("template") {
            None => Some(SmolStr::new_static("default")),
            Some(Value::String(t)) => Some(t.into()),
            _ => return Err("Invalid template"),
        };

        let archive_name = match tab.get("archive_name") {
            None => None,
            Some(Value::String(name)) => Some(name.into()),
            _ => return Err("Invalid archive name"),
        };

        let options = Options::try_from(tab)?;

        Ok(Self {
            template,
            repo,
            archive_name,
            options,
        })
    }
}

#[derive(Debug)]
pub struct Config {
    pub(crate) source: PathBuf,
    pub backups: Backups,
}

impl Config {
    pub fn load<P>(path: &P) -> Result<Self, ConfigError>
    where
        P: AsRef<std::path::Path>,
    {
        use toml::Value;
        let value: Value =
            toml::from_str(&std::fs::read_to_string(path).map_err(ConfigError::IOError)?)
                .map_err(ConfigError::ParseError)?;

        let backups = Backups::try_from(value)?;

        Ok(Self {
            source: path.as_ref().into(),
            backups,
        })
    }
}

#[derive(Clone, Default)]
pub struct Backups(Vec<Archive>);

impl TryFrom<toml::Value> for Backups {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: toml::Value) -> Result<Self, Self::Error> {
        use toml::Value;

        let tab = match value {
            Value::Table(t) => t,
            v => {
                return Err(ConfigError::TypeError {
                    expected: Some("table"),
                    found: Some(v.type_str()),
                })
            }
        };

        let templates = match tab.get("template") {
            None => BTreeMap::<_, PartialBackup>::new(),
            Some(Value::Table(t)) => t
                .into_iter()
                .map(|(k, v)| Ok((k.into(), v.try_into()?)))
                .collect::<Result<_, ConfigError>>()?,
            Some(v) => {
                return Err(ConfigError::TypeError {
                    expected: Some("table"),
                    found: Some(v.type_str()),
                })
            }
        };

        dbg!(&templates);

        let backups = match tab.get("backup") {
            None => Vec::new(),
            Some(Value::Array(a)) => a
                .iter()
                .map(|v| PartialBackup::try_from(v)?.resolve(&templates))
                .collect::<Result<Vec<_>, _>>()?,
            Some(v) => {
                return Err(ConfigError::TypeError {
                    expected: Some("array"),
                    found: Some(v.type_str()),
                })
            }
        };

        dbg!(&backups);

        Ok(Self(backups))
    }
}

impl core::fmt::Debug for Backups {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl core::ops::Deref for Backups {
    type Target = Vec<Archive>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Backups {
    type Item = Archive;
    type IntoIter = std::vec::IntoIter<Archive>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::borg::Compression;

    #[test]
    fn test_empty() {
        let config = "";
        let value: toml::Value = config.parse().unwrap();
        let backups = Backups::try_from(value).unwrap();

        assert!(backups.0.is_empty());
    }

    #[test]
    fn test_defaults() {
        let config = r#"
        [[backup]]
        repository = "."
        "#;

        let value: toml::Value = config.parse().unwrap();
        let backups = Backups::try_from(value).unwrap();

        dbg!(&backups);
        assert_eq!(backups.len(), 1);
        let backup = backups.first().unwrap();
        assert_eq!(backup.repo.to_string(), ".");
        assert_eq!(backup.passphrase(), None);
        assert!(backup.roots().next().is_none());
        assert!(backup.compression().is_none());
        // assert_eq!(archive.pattern_file, None);
        // assert_eq!(archive.exclude_file, Some(PathBuf::from(".borgignore")));
    }

    #[test]
    fn test_template() {
        let config = r#"
        [template.default]
        compression = "lz4"

        [[backup]]
        repository = "."
        "#;

        let value: toml::Value = config.parse().unwrap();
        let backups = Backups::try_from(value).unwrap();

        dbg!(&backups);
        assert_eq!(backups.len(), 1);
        let backup = backups.first().unwrap();
        assert_eq!(backup.compression().unwrap(), Compression::Lz4);
    }

    #[test]
    fn test_custom_template() {
        let config = r#"
        [template.custom]
        compression = "lz4"

        [[backup]]
        template = "custom"
        repository = "."
        "#;

        let value: toml::Value = config.parse().unwrap();
        let backups = Backups::try_from(value).unwrap();

        dbg!(&backups);
        assert_eq!(backups.len(), 1);
        let backup = backups.first().unwrap();
        assert_eq!(backup.compression().unwrap(), Compression::Lz4);
    }
}
