use std::{fmt::Display, num::NonZeroU8, path::PathBuf};

use crate::{Archive, Compression, Passphrase, Repo};

use super::Backup;

#[derive(Debug)]
pub struct Config {
    pub templates: Vec<(String, Template)>,
    pub backups: Vec<Backup>,
}

impl Backup {
    pub(crate) fn apply_template(&mut self, template: &Template) -> &mut Self {
        if self.archive.compression.is_none() {
            self.archive.compression = template.compression.to_owned();
        }

        self
    }
}

#[derive(Debug)]
pub struct Template {
    compression: Option<Compression>,
}

#[derive(Debug)]
pub enum ConfigError {
    TypeError,
    ValueError,
    MissingKey(&'static str),
    ExclusiveKeys(&'static str, &'static str),
    Keyed { key: String, err: Box<ConfigError> },
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
            Self::TypeError => write!(f, "Invalid type"),
            Self::ValueError => write!(f, "Invalid value"),
            Self::MissingKey(key) => write!(f, "Missing key \"{}\"", key),
            Self::ExclusiveKeys(key, other_key) => {
                write!(f, "{} and {} are exclusive", key, other_key)
            }
            Self::Keyed { err, key } => {
                let mut cur = err.to_owned();
                let mut path = vec![key.to_owned()];
                while let ConfigError::Keyed { key, err } = cur.as_ref() {
                    cur = err;
                    path.push(key.to_owned());
                }
                write!(f, "{cur} at {}", path.join("."))
            }
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

impl TryFrom<&toml::Value> for Compression {
    type Error = ConfigError;
    fn try_from(value: &toml::Value) -> Result<Self, Self::Error> {
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
                    _ => return Err(ConfigError::TypeError),
                };
                let level = match t.get("level") {
                    Some(Integer(i)) => Some(*i as u8),
                    None => None,
                    _ => return Err(ConfigError::TypeError),
                };
                let obfuscation = match t.get("obfuscation") {
                    Some(Integer(i)) => Some(
                        NonZeroU8::try_from(*i as u8)
                            .map_err(|_| ConfigError::ValueError.at_key("obfuscation"))?,
                    ),
                    None => None,
                    _ => return Err(ConfigError::TypeError),
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
                    _ => return Err(ConfigError::TypeError.at_key("algorithm")),
                }
            }
            _ => return Err(ConfigError::TypeError),
        };
        Ok(compression)
    }
}

impl TryFrom<&toml::Value> for Template {
    type Error = ConfigError;
    fn try_from(value: &toml::Value) -> Result<Self, Self::Error> {
        use toml::Value::*;
        let table = match value {
            Table(t) => t,
            _ => return Err(ConfigError::TypeError),
        };

        let compression = table
            .get("compression")
            .map(Compression::try_from)
            .transpose()
            .map_err(at_key("compression"))?;

        Ok(Template { compression })
    }
}

impl TryFrom<&toml::map::Map<String, toml::Value>> for Repo {
    type Error = ConfigError;

    fn try_from(value: &toml::map::Map<String, toml::Value>) -> Result<Self, Self::Error> {
        use toml::Value::*;
        let repository = value
            .get("repository")
            .ok_or(ConfigError::MissingKey("repository"))?
            .as_str()
            .ok_or(ConfigError::ValueError.at_key("repository"))?
            .to_owned();

        let passphrase = match (value.get("passphrase"), value.get("passcommand")) {
            (Some(String(p)), None) => Some(Passphrase::Passphrase(p.to_owned())),
            (Some(Integer(fd)), None) => Some(Passphrase::FileDescriptor(fd.to_owned() as i32)),
            (None, Some(String(cmd))) => Some(Passphrase::Command(cmd.to_owned())),
            (Some(_), Some(_)) => {
                return Err(ConfigError::ExclusiveKeys("passphrase", "passcommand"))
            }
            _ => None,
        };

        Ok(Self {
            location: repository,
            passphrase,
        })
    }
}

impl TryFrom<&toml::map::Map<String, toml::Value>> for Archive {
    type Error = ConfigError;

    fn try_from(value: &toml::map::Map<String, toml::Value>) -> Result<Self, Self::Error> {
        use toml::Value::*;

        let compression = value
            .get("compression")
            .map(Compression::try_from)
            .transpose()
            .map_err(|e| e.at_key("compression"))?;

        let comment = match value.get("comment") {
            Some(String(c)) => Some(c.to_owned()),
            None => Some("created using borrg".to_owned()),
            _ => return Err(ConfigError::TypeError.at_key("comment")),
        };

        let exclude_file = match value.get("exclude_file") {
            Some(String(p)) => Some(PathBuf::from(p)),
            None => Some(PathBuf::from(".borgignore")),
            _ => return Err(ConfigError::TypeError.at_key("exclude_file")),
        };

        let pattern_file = match value.get("pattern_file") {
            Some(String(p)) => Some(PathBuf::from(p)),
            None => None,
            _ => return Err(ConfigError::TypeError.at_key("pattern_file")),
        };

        let name = chrono::Local::now().format("%Y-%m-%d").to_string();

        let paths = match value.get("paths") {
            Some(Array(a)) => a
                .into_iter()
                .enumerate()
                .map(|(i, p)| {
                    let key = i.to_string();
                    Ok(p.as_str()
                        .ok_or(ConfigError::TypeError.at_key(&key))?
                        .into())
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => vec![PathBuf::from("~")],
            _ => return Err(ConfigError::TypeError.at_key("paths")),
        };

        Ok(Self {
            comment,
            compression,
            exclude_file,
            name,
            paths,
            pattern_file,
        })
    }
}

impl TryFrom<&toml::Value> for Backup {
    type Error = ConfigError;
    fn try_from(value: &toml::Value) -> Result<Self, Self::Error> {
        use toml::Value::*;
        let table = value.as_table().ok_or(ConfigError::TypeError)?;

        let repo = Repo::try_from(table)?;
        let archive = Archive::try_from(table)?;
        let template = match table.get("template") {
            Some(String(t)) => t.to_owned(),
            Some(_) => return Err(ConfigError::TypeError.at_key("template")),
            None => "default".to_owned(),
        };

        Ok(Self {
            repo,
            archive,
            template,
        })
    }
}

impl TryFrom<toml::Value> for Config {
    type Error = ConfigError;
    fn try_from(value: toml::Value) -> Result<Self, Self::Error> {
        use toml::Value::*;

        let table = value.as_table().ok_or(ConfigError::TypeError)?;

        let templates = match table.get("template") {
            Some(Table(t)) => t
                .into_iter()
                .map(|(k, v)| Ok((k.to_owned(), Template::try_from(v).map_err(at_key(k))?)))
                .collect::<Result<Vec<_>, ConfigError>>()
                .map_err(at_key("template"))?,
            Some(_) => return Err(ConfigError::TypeError.at_key("template")),
            None => vec![],
        };

        let backups = match table.get("backup") {
            Some(Array(b)) => b,
            Some(_) => return Err(ConfigError::TypeError.at_key("backup")),
            None => return Err(ConfigError::MissingKey("backup")),
        };

        let backups = backups
            .into_iter()
            .enumerate()
            .map(|(i, b)| {
                Backup::try_from(b)
                    .map(|mut b| {
                        for (k, t) in templates.iter() {
                            if k == &b.template {
                                b.apply_template(&t);
                                break;
                            }
                        }
                        b
                    })
                    .map_err(at_key(i.to_string()))
                    .map_err(at_key("backup"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { templates, backups })
    }
}
