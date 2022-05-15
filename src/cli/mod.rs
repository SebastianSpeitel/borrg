use std::{num::NonZeroU8, path::PathBuf, sync::mpsc, time::Duration};

use clap::Args;
// mod create;
mod run;
// use crate::{wrapper::BorgWrapper, Backend, Event};
// pub use create::*;
use log::debug;
pub use run::*;

use crate::{Archive, Borg, Compression, Error, Passphrase, Repo, Result};

#[inline]
pub fn resolve_path(path: &PathBuf) -> PathBuf {
    if path == &PathBuf::from("~") {
        return dirs::home_dir().unwrap();
    }

    match path.strip_prefix("~/") {
        Ok(path) => dirs::home_dir().unwrap().join(path),
        Err(_) => path.to_owned(),
    }

    // match path.strip_prefix("~") {
    //     // Path starts with "~"
    //     Ok(p) => match p.strip_prefix("/") {
    //         // Path starts with "~/"
    //         Ok(path) => dirs::home_dir().unwrap().join(path),
    //         // Filename starts with "~"
    //         Err(..) => path.to_owned(),
    //     },
    //     // Path doesn't start with "~"
    //     Err(_) => path.to_owned(),
    // }
}

#[derive(Debug)]
pub struct Backup {
    pub name: String,
    pub repo: Repo,
    pub archive: Archive,
}

impl TryFrom<toml::Value> for Compression {
    type Error = Error;
    fn try_from(value: toml::Value) -> Result<Self> {
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
                _ => return Err("compression is not a valid value".into()),
            },
            Table(t) => {
                let auto = match t.get("auto") {
                    Some(Boolean(b)) => *b,
                    None => false,
                    _ => return Err("auto is not a boolean".into()),
                };
                let level = match t.get("level") {
                    Some(Integer(i)) => Some(*i as u8),
                    None => None,
                    _ => return Err("level is not an integer".into()),
                };
                let obfuscation = match t.get("obfuscation") {
                    Some(Integer(i)) => Some(NonZeroU8::try_from(*i as u8)?),
                    None => None,
                    _ => return Err("obfuscation is not an integer".into()),
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
                        _ => return Err(format!("invalid algorithm: {}", s).into()),
                    },
                    None => return Err("no algorithm specified".into()),
                    _ => return Err("algorithm is not a string".into()),
                }
            }
            _ => return Err("compression is not a string or table".into()),
        };
        Ok(compression)
    }
}

#[derive(Debug)]
pub struct Config {
    pub backups: Vec<Backup>,
}

impl TryFrom<toml::Value> for Config {
    type Error = Error;
    fn try_from(value: toml::Value) -> Result<Self> {
        let backups = get_backups(&value)?;
        Ok(Config { backups })
    }
}

pub fn get_backups(config: &toml::Value) -> Result<Vec<Backup>> {
    use toml::map::Map;
    use toml::Value::*;

    match config {
        Table(t) => {
            let default = t.get("default");
            let default = match default {
                Some(Table(t)) => t.to_owned(),
                None => Map::new(),
                _ => return Err("default is not a table".into()),
            };

            // let backups = t.get("backup");
            // let backup_configs = match backups {
            //     Some(Table(b)) => b,
            //     None => return Err("no backups in config".into()),
            //     _ => return Err("backup is not an array".into()),
            // };
            let mut backups = Vec::new();

            for (backup, config) in t.iter() {
                // Skip default backup
                if backup == "default" {
                    continue;
                }

                let repository = config
                    .get("repository")
                    .or_else(|| default.get("repository"));
                let mut repo: Repo = match repository {
                    Some(String(s)) => s.to_owned().into(),
                    Some(_) => return Err("repository is not a string".into()),
                    _ => return Err("no repository configured".into()),
                };

                let name = config.get("name").or_else(|| default.get("name"));
                let name = match name {
                    Some(String(s)) => Some(s.to_owned()),
                    Some(_) => return Err("name is not a string".into()),
                    _ => None,
                };

                let mut archive = match name {
                    Some(name) => Archive::new(name.to_owned()),
                    None => Archive::today(),
                };

                let compression = config
                    .get("compression")
                    .or_else(|| default.get("compression"))
                    .map(|c| Compression::try_from(c.to_owned()));
                match compression {
                    Some(Ok(c)) => {
                        archive.compression(c);
                    }
                    Some(Err(e)) => {
                        return Err(e);
                    }
                    None => {}
                }

                match config
                    .get("passphrase")
                    .or_else(|| default.get("passphrase"))
                {
                    Some(String(s)) => {
                        repo.passphrase(Passphrase::Passphrase(s.to_owned()));
                    }
                    Some(Integer(fd)) => {
                        repo.passphrase(Passphrase::FileDescriptor(*fd as i32));
                    }
                    Some(_) => return Err("passphrase is not a string".into()),
                    _ => {}
                }
                match config
                    .get("passcommand")
                    .or_else(|| default.get("passcommand"))
                {
                    Some(String(s)) => {
                        repo.passphrase(Passphrase::Command(s.to_owned()));
                    }
                    Some(_) => return Err("passcommand is not a string".into()),
                    _ => {}
                }

                let paths = config.get("path").or_else(|| default.get("path"));
                match paths {
                    Some(Array(p)) => {
                        for path in p {
                            if let String(s) = path {
                                archive.path(PathBuf::from(s));
                            } else {
                                return Err("path is not a string".into());
                            }
                        }
                    }
                    Some(String(p)) => {
                        archive.path(p.into());
                    }
                    Some(_) => return Err("path is not an array or string".into()),
                    _ => {
                        let home_dir = match dirs::home_dir() {
                            Some(h) => h,
                            None => return Err("no home directory".into()),
                        };
                        archive.path(home_dir);
                    }
                };

                let pattern_file = config
                    .get("pattern_file")
                    .or_else(|| default.get("pattern_file"));
                match pattern_file {
                    Some(String(s)) => {
                        archive.pattern_file(PathBuf::from(s));
                    }
                    Some(_) => return Err("pattern_file is not a string".into()),
                    _ => {}
                };

                let exclude_file = config
                    .get("exclude_file")
                    .or_else(|| default.get("exclude_file"));
                match exclude_file {
                    Some(String(s)) => {
                        archive.exclude_file(PathBuf::from(s));
                    }
                    Some(_) => return Err("exclude_file is not a string".into()),
                    _ => {
                        archive.exclude_file(PathBuf::from(".borgignore"));
                    }
                };

                backups.push(Backup {
                    name: backup.to_owned(),
                    repo,
                    archive,
                });
            }
            debug!("Backups: {:#?}", &backups);
            Ok(backups)
        }
        _ => Err("config is not a table".into()),
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
