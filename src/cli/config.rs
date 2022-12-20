use std::{fmt::Display, num::NonZeroU8, path::PathBuf};

use log::{debug, warn};

use crate::{Archive, Compression, Passphrase, Repo};

#[derive(Debug)]
pub enum ConfigError {
    TypeError {
        expected: Option<&'static str>,
        found: Option<&'static str>,
    },
    ValueError,
    MissingKey(&'static str),
    ExclusiveKeys(&'static str, &'static str),
    MissingTemplate(String),
    Keyed {
        key: String,
        err: Box<ConfigError>,
    },
    IOError(std::io::Error),
    ParseError(toml::de::Error),
    Other(&'static str),
}

impl ConfigError {
    fn at_key<T: AsRef<str>>(self, key: T) -> ConfigError {
        ConfigError::Keyed {
            key: key.as_ref().to_string(),
            err: Box::new(self),
        }
    }
}

fn at_key<T: AsRef<str>>(key: T) -> impl FnOnce(ConfigError) -> ConfigError {
    move |err: ConfigError| ConfigError::Keyed {
        key: key.as_ref().to_string(),
        err: Box::new(err),
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TypeError {
                expected: None,
                found: None,
            } => write!(f, "Invalid type"),
            Self::TypeError {
                expected: Some(expected),
                found: Some(received),
            } => write!(f, "Invalid type: expected {}, found {}", expected, received),
            Self::TypeError {
                expected: Some(expected),
                found: None,
            } => write!(f, "Invalid type: expected {}", expected),
            Self::TypeError {
                expected: None,
                found: Some(received),
            } => write!(f, "Invalid type: found {}", received),
            Self::ValueError => write!(f, "Invalid value"),
            Self::MissingKey(key) => write!(f, "Missing key \"{}\"", key),
            Self::ExclusiveKeys(key, other_key) => {
                write!(f, "{} and {} are exclusive", key, other_key)
            }
            Self::MissingTemplate(name) => write!(f, "Missing template \"{}\"", name),
            Self::Keyed { err, key } => {
                let mut cur = err.to_owned();
                let mut path = vec![key.to_owned()];
                while let ConfigError::Keyed { key, err } = cur.as_ref() {
                    cur = err;
                    path.push(key.to_owned());
                }
                write!(f, "{cur} at {}", path.join("."))
            }
            Self::IOError(err) => err.fmt(f),
            Self::ParseError(err) => err.fmt(f),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl std::process::Termination for ConfigError {
    fn report(self) -> std::process::ExitCode {
        eprintln!("{}", self);
        std::process::ExitCode::FAILURE
    }
}

#[derive(Clone, Debug)]
enum RepoConfig {
    Split {
        user: Option<String>,
        host: Option<String>,
        path: Option<PathBuf>,
    },
    Combined(String),
}

impl RepoConfig {
    pub fn inherit(&mut self, other: &RepoConfig) {
        use RepoConfig::*;
        // inherit user
        if let Split { user, .. } = self {
            if user.is_none() {
                if let Split { user: u, .. } = other {
                    *user = u.clone();
                }
            }
        }
        // inherit host
        if let Split { host, .. } = self {
            if host.is_none() {
                if let Split { host: h, .. } = other {
                    *host = h.clone();
                }
            }
        }
        // inherit path
        if let Split { path, .. } = self {
            if path.is_none() {
                if let Split { path: p, .. } = other {
                    *path = p.clone();
                }
            }
        }
    }
}

impl Display for RepoConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoConfig::Split {
                user: None,
                host: Some(h),
                path: None,
            } => write!(f, "{h}"),
            RepoConfig::Split {
                user: None,
                host: None,
                path: Some(p),
            } => write!(f, "{}", p.display()),
            RepoConfig::Split {
                user: None,
                host: Some(h),
                path: Some(p),
            } => write!(f, "{h}:{}", p.display()),
            RepoConfig::Split {
                user: Some(u),
                host: Some(h),
                path: Some(p),
            } => write!(f, "{u}@{h}:{}", p.display()),
            RepoConfig::Combined(combined) => write!(f, "{}", combined),
            _ => {
                warn!("RepoConfig::fmt: Unhandled case");
                write!(f, "::")
            }
        }
    }
}

/// Configuration for a backup
///
/// All fields are optional, because they can be inherited.
#[derive(Debug)]
struct BackupConfig {
    /// Name of template to inherit from
    pub template: Option<String>,

    /// Repository to backup to
    pub repo: Option<RepoConfig>,

    /// Passphrase
    pub passphrase: Option<Passphrase>,

    /// Paths to backup
    ///
    /// To inherit from a template, use `...` as path.
    pub paths: Vec<PathBuf>,

    /// Compression level
    pub compression: Option<Compression>,

    /// Pattern file
    pub pattern_file: Option<PathBuf>,

    /// Exclude file
    pub exclude_file: Option<PathBuf>,
}

impl BackupConfig {
    pub fn set_defaults(&mut self) {
        self.template = None;
        self.resolve_with(&Default::default());
    }

    pub fn resolve(mut self, templates: &[(String, BackupConfig)]) -> Result<Self, ConfigError> {
        while let Some(t) = self.template.take() {
            let template =
                templates.iter().find_map(
                    |(name, template)| {
                        if name == &t {
                            Some(template)
                        } else {
                            None
                        }
                    },
                );
            if let Some(template) = template {
                self.resolve_with(template);
            } else {
                return Err(ConfigError::MissingTemplate(t));
            }
        }

        Ok(self)
    }

    pub fn resolve_with(&mut self, template: &Self) {
        // Inherit template
        self.template = template.template.to_owned();

        // Merge repo
        match self.repo {
            None => self.repo = template.repo.clone(),
            Some(RepoConfig::Combined(_)) => {}
            Some(ref mut r) => {
                if let Some(t) = &template.repo {
                    r.inherit(t)
                }
            }
        };

        // Inherit passphrase
        if self.passphrase.is_none() {
            self.passphrase = template.passphrase.to_owned();
        }

        // Inherit path if empty otherwise replace "..." with paths from template
        if self.paths.is_empty() {
            self.paths = template.paths.clone();
        } else {
            self.paths = self
                .paths
                .iter()
                .flat_map(|path| {
                    if path.as_os_str() == "..." {
                        template.paths.clone()
                    } else {
                        vec![path.clone()]
                    }
                })
                .collect();
        }

        // Inherit compression
        if self.compression.is_none() {
            self.compression = template.compression.to_owned();
        }

        // Inherit pattern file
        if self.pattern_file.is_none() {
            self.pattern_file = template.pattern_file.to_owned();
        }

        // Inherit exclude file
        if self.exclude_file.is_none() {
            self.exclude_file = template.exclude_file.to_owned();
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        BackupConfig {
            template: None,
            repo: None,
            passphrase: None,
            paths: vec![PathBuf::from("~")],
            compression: None,
            pattern_file: None,
            exclude_file: Some(PathBuf::from(".borgignore")),
        }
    }
}

impl ConfigProperty for Compression {
    fn parse(value: &toml::Value) -> Result<Self, ConfigError> {
        use toml::Value::*;
        let compression = match value {
            String(s) => match s.to_lowercase().as_str() {
                "none" => Compression::None { obfuscation: None },
                "lz4" => Compression::Lz4 {
                    auto: false,
                    obfuscation: None,
                },
                "lzma" => Compression::Lzma {
                    level: None,
                    auto: false,
                    obfuscation: None,
                },
                "zlib" => Compression::Zlib {
                    level: None,
                    auto: false,
                    obfuscation: None,
                },
                "zstd" => Compression::Zstd {
                    level: None,
                    auto: false,
                    obfuscation: None,
                },
                _ => return Err(ConfigError::ValueError),
            },
            Table(t) => {
                let auto = match t.get("auto") {
                    Some(Boolean(b)) => *b,
                    None => false,
                    _ => {
                        return Err(ConfigError::TypeError {
                            expected: Some("boolean"),
                            found: Some(value.type_str()),
                        }
                        .at_key("auto"))
                    }
                };
                let level = match t.get("level") {
                    Some(Integer(i)) => Some(*i as u8),
                    None => None,
                    _ => {
                        return Err(ConfigError::TypeError {
                            expected: Some("integer"),
                            found: Some(value.type_str()),
                        }
                        .at_key("level"))
                    }
                };
                let obfuscation = match t.get("obfuscation") {
                    Some(Integer(i)) => Some(
                        NonZeroU8::try_from(*i as u8)
                            .map_err(|_| ConfigError::ValueError.at_key("obfuscation"))?,
                    ),
                    None => None,
                    _ => {
                        return Err(ConfigError::TypeError {
                            expected: Some("integer"),
                            found: Some(value.type_str()),
                        }
                        .at_key("obfuscation"))
                    }
                };
                match t.get("algorithm") {
                    Some(String(s)) => match s.to_lowercase().as_str() {
                        "none" => Compression::None { obfuscation },
                        "lz4" => Compression::Lz4 { auto, obfuscation },
                        "zstd" => Compression::Zstd {
                            level,
                            auto,
                            obfuscation,
                        },
                        "zlib" => Compression::Zlib {
                            level,
                            auto,
                            obfuscation,
                        },
                        "lzma" => Compression::Lzma {
                            level,
                            auto,
                            obfuscation,
                        },
                        _ => return Err(ConfigError::ValueError.at_key("algorithm")),
                    },
                    None => return Err(ConfigError::MissingKey("algorithm")),
                    _ => {
                        return Err(ConfigError::TypeError {
                            expected: Some("string"),
                            found: Some(value.type_str()),
                        }
                        .at_key("algorithm"))
                    }
                }
            }
            _ => {
                return Err(ConfigError::TypeError {
                    expected: Some("string or table"),
                    found: Some(value.type_str()),
                })
            }
        };
        Ok(compression)
    }
}

impl TryFrom<&BackupConfig> for Repo {
    type Error = ConfigError;
    fn try_from(config: &BackupConfig) -> Result<Self, Self::Error> {
        let repository = config
            .repo
            .as_ref()
            .ok_or(ConfigError::MissingKey("repo"))?
            .to_string();

        let mut repo = repository.parse::<Repo>().map_err(ConfigError::Other)?;

        repo.passphrase = config.passphrase.to_owned();

        Ok(repo)
    }
}

impl TryFrom<&BackupConfig> for Archive {
    type Error = ConfigError;
    fn try_from(config: &BackupConfig) -> Result<Self, Self::Error> {
        let name = chrono::Local::now().format("%Y-%m-%d").to_string();

        let paths = if config.paths.is_empty() {
            return Err(ConfigError::MissingKey("path"));
        } else {
            config.paths.clone()
        };

        let compression = config.compression.to_owned();
        let pattern_file = config.pattern_file.to_owned();
        let exclude_file = config.exclude_file.to_owned();

        Ok(Self {
            name,
            paths,
            compression,
            pattern_file,
            exclude_file,
            comment: None,
        })
    }
}

impl TryFrom<BackupConfig> for (Repo, Archive) {
    type Error = ConfigError;
    fn try_from(config: BackupConfig) -> Result<Self, ConfigError> {
        Ok((Repo::try_from(&config)?, Archive::try_from(&config)?))
    }
}

trait ConfigProperty
where
    Self: Sized,
{
    fn parse(value: &toml::Value) -> Result<Self, ConfigError>;

    fn from_map(
        map: &toml::map::Map<String, toml::Value>,
        key: &str,
    ) -> Result<Option<Self>, ConfigError> {
        map.get(key)
            .map(Self::parse)
            .transpose()
            .map_err(at_key(key))
    }
}

impl ConfigProperty for String {
    fn parse(value: &toml::Value) -> Result<Self, ConfigError> {
        match value {
            toml::Value::String(s) => Ok(s.to_owned()),
            _ => Err(ConfigError::TypeError {
                expected: Some("string"),
                found: Some(value.type_str()),
            }),
        }
    }
}

impl ConfigProperty for PathBuf {
    fn parse(value: &toml::Value) -> Result<Self, ConfigError> {
        match value {
            toml::Value::String(s) => Ok(PathBuf::from(s)),
            _ => Err(ConfigError::TypeError {
                expected: Some("string"),
                found: Some(value.type_str()),
            }),
        }
    }
}

impl ConfigProperty for RepoConfig {
    fn parse(value: &toml::Value) -> Result<Self, ConfigError> {
        match value {
            toml::Value::String(s) => Ok(RepoConfig::Combined(s.to_owned())),
            toml::Value::Table(t) => {
                let user: Option<String> = ConfigProperty::from_map(t, "user")?;
                let host: Option<String> = ConfigProperty::from_map(t, "host")?;
                let path: Option<PathBuf> = ConfigProperty::from_map(t, "path")?;

                Ok(RepoConfig::Split { user, host, path })
            }
            _ => Err(ConfigError::TypeError {
                expected: Some("string or table"),
                found: Some(value.type_str()),
            }),
        }
    }
}

impl<T> ConfigProperty for Vec<T>
where
    T: ConfigProperty,
{
    fn parse(value: &toml::Value) -> Result<Self, ConfigError> {
        if let Ok(val) = T::parse(value) {
            return Ok(vec![val]);
        }
        match value {
            toml::Value::Array(a) => a.iter().map(T::parse).collect(),
            _ => Err(ConfigError::TypeError {
                expected: Some("array"),
                found: Some(value.type_str()),
            }),
        }
    }
}

impl<T> ConfigProperty for Vec<(String, T)>
where
    T: ConfigProperty,
{
    fn parse(value: &toml::Value) -> Result<Self, ConfigError> {
        match value {
            toml::Value::Table(t) => t
                .iter()
                .map(|(k, v)| Ok((k.to_owned(), T::parse(v)?)))
                .collect(),
            _ => Err(ConfigError::TypeError {
                expected: Some("table"),
                found: Some(value.type_str()),
            }),
        }
    }
}

impl ConfigProperty for BackupConfig {
    fn parse(value: &toml::Value) -> Result<Self, ConfigError> {
        use toml::Value as T;

        let map = value.as_table().ok_or(ConfigError::TypeError {
            expected: Some("table"),
            found: Some(value.type_str()),
        })?;

        let template: String =
            ConfigProperty::from_map(map, "template")?.unwrap_or_else(|| "default".to_string());

        let repo: Option<RepoConfig> = ConfigProperty::from_map(map, "repository")?;

        let passphrase = match (map.get("passphrase"), map.get("passcommand")) {
            (Some(T::String(p)), None) => Some(Passphrase::Passphrase(p.to_owned())),
            (Some(T::Integer(fd)), None) => Some(Passphrase::FileDescriptor(fd.to_owned() as i32)),
            (None, Some(T::String(cmd))) => Some(Passphrase::Command(cmd.to_owned())),
            (Some(_), Some(_)) => {
                return Err(ConfigError::ExclusiveKeys("passphrase", "passcommand"))
            }
            _ => None,
        };

        let paths: Vec<PathBuf> = ConfigProperty::from_map(map, "path")?.unwrap_or_default();

        let compression: Option<Compression> = ConfigProperty::from_map(map, "compression")?;

        let pattern_file: Option<PathBuf> = ConfigProperty::from_map(map, "pattern_file")?;

        let exclude_file: Option<PathBuf> = ConfigProperty::from_map(map, "exclude_file")?;

        Ok(Self {
            template: Some(template),
            repo,
            passphrase,
            paths,
            compression,
            pattern_file,
            exclude_file,
        })
    }
}

impl ConfigProperty for Vec<(Repo, Archive)> {
    fn parse(value: &toml::Value) -> Result<Self, ConfigError> {
        let map = value.as_table().ok_or(ConfigError::TypeError {
            expected: Some("table"),
            found: Some(value.type_str()),
        })?;

        let templates: Vec<(String, BackupConfig)> =
            ConfigProperty::from_map(map, "template")?.unwrap_or_default();

        // Set default values in default tepmplate
        let mut has_default_template = false;
        let mut templates = templates
            .into_iter()
            .map(|(n, mut c)| {
                if n == "default" {
                    has_default_template = true;
                    c.set_defaults();
                }
                (n, c)
            })
            .collect::<Vec<_>>();

        if !has_default_template {
            templates.push(("default".to_string(), BackupConfig::default()));
        }

        let backups: Vec<BackupConfig> =
            ConfigProperty::from_map(map, "backup")?.unwrap_or_default();

        debug!("Parsed templates: {:#?}", templates);
        debug!("Parsed backups: {:#?}", backups);

        backups
            .into_iter()
            .map(|c| c.resolve(&templates)?.try_into())
            .collect()
    }
}

#[derive(Debug)]
pub struct Config {
    pub backups: Vec<(Repo, Archive)>,
}

impl Config {
    pub fn load<P>(path: &P) -> Result<Self, ConfigError>
    where
        P: AsRef<std::path::Path>,
    {
        let value = toml::from_str(&std::fs::read_to_string(path).map_err(ConfigError::IOError)?)
            .map_err(ConfigError::ParseError)?;

        let backups = ConfigProperty::parse(&value)?;

        Ok(Self { backups })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_empty() {
        let config = "";
        let value = config.parse().unwrap();
        let result: Result<Vec<(Repo, Archive)>, ConfigError> = ConfigProperty::parse(&value);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_defaults() {
        let config = r#"
        [[backup]]
        repository = "."
        "#;

        let value = config.parse().unwrap();
        let result: Result<Vec<(Repo, Archive)>, ConfigError> = ConfigProperty::parse(&value);

        dbg!(&result);
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        let (repo, archive) = results.first().unwrap();
        assert_eq!(repo.to_string(), ".");
        assert_eq!(repo.passphrase, None);
        assert_eq!(archive.paths, vec![PathBuf::from("~")]);
        assert_eq!(archive.compression, None);
        assert_eq!(archive.pattern_file, None);
        assert_eq!(archive.exclude_file, Some(PathBuf::from(".borgignore")));
    }

    #[test]
    fn test_template() {
        let config = r#"
        [template.default]
        compression = "lz4"

        [[backup]]
        repository = "."
        "#;

        let value = config.parse().unwrap();
        let result: Result<Vec<(Repo, Archive)>, ConfigError> = ConfigProperty::parse(&value);

        dbg!(&result);
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        let (_, archive) = results.first().unwrap();
        assert!(matches!(archive.compression, Some(Compression::Lz4 { .. })));
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

        let value = config.parse().unwrap();
        let result: Result<Vec<(Repo, Archive)>, ConfigError> = ConfigProperty::parse(&value);

        dbg!(&result);
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        let (_, archive) = results.first().unwrap();
        assert!(matches!(archive.compression, Some(Compression::Lz4 { .. })));
    }
}
