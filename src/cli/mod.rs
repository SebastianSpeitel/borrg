use std::{
    fmt::{Debug, Display},
    path::PathBuf,
    sync::mpsc,
    time::Duration,
};

use clap::Args;
// mod create;
mod config;
pub mod run;
pub use config::Config;
// use crate::{wrapper::BorgWrapper, Backend, Event};
// pub use create::*;

use crate::{Archive, Borg, Repo};

#[derive(Debug)]
pub struct Backup {
    pub repo: Repo,
    pub archive: Archive,
    pub template: String,
}

impl Display for Backup {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}::{}", self.repo.location, self.archive.name)
    }
}

// impl TryFrom<toml::Value> for Repo{
//     type Error = ConfigError;
//     fn try_from(value: toml::Value) -> std::result::Result<Self,Self::Error> {
//         unimplemented!()
//     }
// }

// #[derive(Debug)]
// pub struct Config {
//     pub backups: Vec<Backup>,
// }

// impl TryFrom<toml::Value> for Config {
//     type Error = Error;
//     fn try_from(value: toml::Value) -> Result<Self> {
//         let backups = get_backups(&value)?;
//         Ok(Config { backups })
//     }
// }

// pub fn get_backups(config: &toml::Value) -> Result<Vec<Backup>> {
//     use toml::map::Map;
//     use toml::Value::*;

//     match config {
//         Table(t) => {
//             let default = t.get("default");
//             let default = match default {
//                 Some(Table(t)) => t.to_owned(),
//                 None => Map::new(),
//                 _ => return Err("default is not a table".into()),
//             };

//             // let backups = t.get("backup");
//             // let backup_configs = match backups {
//             //     Some(Table(b)) => b,
//             //     None => return Err("no backups in config".into()),
//             //     _ => return Err("backup is not an array".into()),
//             // };
//             let mut backups = Vec::new();

//             let backup_array = match t.get("backup") {
//                 Some(Array(a)) => a,
//                 None => return Err("no backups in config".into()),
//                 _ => return Err("backup is not an array".into()),
//             };

//             for backup in backup_array {
//                 let repository = backup
//                     .get("repository")
//                     .or_else(|| default.get("repository"));
//                 let mut repo: Repo = match repository {
//                     Some(String(s)) => s.to_owned().into(),
//                     Some(_) => return Err("repository is not a string".into()),
//                     _ => return Err("no repository configured".into()),
//                 };

//                 let name = backup.get("name").or_else(|| default.get("name"));
//                 let name = match name {
//                     Some(String(s)) => Some(s.to_owned()),
//                     Some(_) => return Err("name is not a string".into()),
//                     _ => None,
//                 };

//                 let mut archive = match name {
//                     Some(name) => Archive::new(name.to_owned()),
//                     None => Archive::today(),
//                 };

//                 let compression = backup
//                     .get("compression")
//                     .or_else(|| default.get("compression"))
//                     .map(|c| Compression::try_from(c.to_owned()));
//                 match compression {
//                     Some(Ok(c)) => {
//                         archive.compression(c);
//                     }
//                     Some(Err(e)) => {
//                         return Err(e);
//                     }
//                     None => {}
//                 }

//                 match backup
//                     .get("passphrase")
//                     .or_else(|| default.get("passphrase"))
//                 {
//                     Some(String(s)) => {
//                         repo.passphrase(Passphrase::Passphrase(s.to_owned()));
//                     }
//                     Some(Integer(fd)) => {
//                         repo.passphrase(Passphrase::FileDescriptor(*fd as i32));
//                     }
//                     Some(_) => return Err("passphrase is not a string".into()),
//                     _ => {}
//                 }
//                 match backup
//                     .get("passcommand")
//                     .or_else(|| default.get("passcommand"))
//                 {
//                     Some(String(s)) => {
//                         repo.passphrase(Passphrase::Command(s.to_owned()));
//                     }
//                     Some(_) => return Err("passcommand is not a string".into()),
//                     _ => {}
//                 }

//                 let paths = backup.get("path").or_else(|| default.get("path"));
//                 match paths {
//                     Some(Array(p)) => {
//                         for path in p {
//                             if let String(s) = path {
//                                 archive.path(PathBuf::from(s));
//                             } else {
//                                 return Err("path is not a string".into());
//                             }
//                         }
//                     }
//                     Some(String(p)) => {
//                         archive.path(p.into());
//                     }
//                     Some(_) => return Err("path is not an array or string".into()),
//                     _ => {
//                         let home_dir = match dirs::home_dir() {
//                             Some(h) => h,
//                             None => return Err("no home directory".into()),
//                         };
//                         archive.path(home_dir);
//                     }
//                 };

//                 let pattern_file = backup
//                     .get("pattern_file")
//                     .or_else(|| default.get("pattern_file"));
//                 match pattern_file {
//                     Some(String(s)) => {
//                         archive.pattern_file(PathBuf::from(s));
//                     }
//                     Some(_) => return Err("pattern_file is not a string".into()),
//                     _ => {}
//                 };

//                 let exclude_file = backup
//                     .get("exclude_file")
//                     .or_else(|| default.get("exclude_file"));
//                 match exclude_file {
//                     Some(String(s)) => {
//                         archive.exclude_file(PathBuf::from(s));
//                     }
//                     Some(_) => return Err("exclude_file is not a string".into()),
//                     _ => {
//                         archive.exclude_file(PathBuf::from(".borgignore"));
//                     }
//                 };

//                 backups.push(Backup { repo, archive });
//             }
//             debug!("Backups: {:#?}", &backups);
//             Ok(backups)
//         }
//         _ => Err("config is not a table".into()),
//     }
// }
