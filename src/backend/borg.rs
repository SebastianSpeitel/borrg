use crate::borrg::*;
use log::{debug, warn, Level};
use std::{
    io::{BufRead, BufReader, Lines, Read},
    path::PathBuf,
    process::{ChildStderr, Command, Stdio},
    time::{Duration, SystemTime},
};

impl TryFrom<serde_json::Value> for Event {
    type Error = Error;
    fn try_from(value: serde_json::Value) -> Result<Self> {
        let _type = match value.get("type") {
            Some(serde_json::Value::String(t)) => t,
            _ => Err("no type")?,
        };

        let time = || {
            value
                .get("time")
                .and_then(|t| t.as_f64())
                .and_then(|t| SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs_f64(t)))
        };

        let nfiles = || value.get("nfiles").and_then(|n| n.as_u64());
        let compressed_size = || value.get("compressed_size").and_then(|s| s.as_u64());
        let deduplicated_size = || value.get("deduplicated_size").and_then(|s| s.as_u64());
        let original_size = || value.get("original_size").and_then(|s| s.as_u64());
        let path = || {
            value
                .get("path")
                .and_then(|p| p.as_str())
                .map(|p| PathBuf::from(p))
        };
        let message = || {
            value
                .get("message")
                .and_then(|m| m.as_str())
                .map(|m| m.to_owned())
        };
        let finished = || value.get("finished").and_then(|f| f.as_bool());
        let msgid = || {
            value
                .get("msgid")
                .and_then(|m| m.as_str())
                .map(|m| m.to_owned())
        };
        let operation = || value.get("operation").and_then(|o| o.as_u64());
        let level = || {
            value
                .get("level")
                .and_then(|l| l.as_str())
                .and_then(|l| match l {
                    "debug" => Some(Level::Debug),
                    "info" => Some(Level::Info),
                    "warning" => Some(Level::Warn),
                    "error" => Some(Level::Error),
                    _ => {
                        warn!("unknown log level: {}", l);
                        None
                    }
                })
        };
        let name = || {
            value
                .get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.to_owned())
        };
        let status = || {
            value
                .get("status")
                .and_then(|s| s.as_str())
                .map(|s| s.to_owned())
        };

        let event = match _type.as_str() {
            "archive_progress" => Self::ArchiveProgress {
                nfiles: nfiles().unwrap_or_default(),
                compressed_size: compressed_size().unwrap_or_default(),
                deduplicated_size: deduplicated_size().unwrap_or_default(),
                original_size: original_size().unwrap_or_default(),
                path: path().unwrap_or_default().to_owned(),
                time: time(),
            },
            "progress_message" => Self::ProgressMessage {
                message: message(),
                finished: finished(),
                msgid: msgid(),
                operation: operation(),
                time: time(),
            },
            "log_message" => Self::LogMessage {
                name: name(),
                level: level(),
                message: message().unwrap_or_default().to_owned(),
                msgid: msgid(),
                time: time(),
            },
            "file_status" => Self::FileStatus {
                path: path().unwrap_or_default().to_owned(),
                status: status().unwrap_or_default().to_owned(),
            },
            _ => Err(format!("Unknown event type: {}", _type))?,
        };
        Ok(event)
    }
}

pub struct Events<R: Read> {
    lines: Lines<BufReader<R>>,
}

impl<R: Read> From<R> for Events<R> {
    fn from(readable: R) -> Self {
        Events {
            lines: BufReader::new(readable).lines(),
        }
    }
}

impl<R: Read> Iterator for Events<R> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?;
        let line = match line {
            Ok(line) => line,
            Err(err) => return Some(Event::Error(Box::new(err))),
        };

        let json: std::result::Result<serde_json::Value, _> = serde_json::from_str(&line);
        let json = match json {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to parse JSON: {}", e);
                return Some(Event::Error(Box::new(e)));
            }
        };

        debug!("{:#?}", json);

        match Event::try_from(json) {
            Ok(event) => Some(event),
            Err(e) => {
                warn!("Failed to parse event: {}", e);
                Some(Event::Error(
                    Box::<dyn std::error::Error + Send + Sync>::from(e),
                ))
            }
        }
    }
}

fn log_command(cmd: &Command) {
    let command = format!(
        "{} {}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args()
            .map(|a| a.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );
    debug!("Executing command: {}", command);
}

impl TryFrom<serde_json::Value> for RepoInfo {
    type Error = Error;
    fn try_from(value: serde_json::Value) -> Result<Self> {
        let cache = value
            .get("cache")
            .and_then(|c| c.as_object())
            .ok_or("missing key: \"cache\"")?;
        let cache_path = cache
            .get("path")
            .and_then(|p| p.as_str())
            .map(|p| PathBuf::from(p))
            .ok_or("missing key: \"cache.path\"")?;
        let stats = cache
            .get("stats")
            .and_then(|s| s.as_object())
            .ok_or("missing key: \"cache.stats\"")?;
        let total_chunks = stats
            .get("total_chunks")
            .and_then(|t| t.as_u64())
            .ok_or("missing key: \"cache.stats.total_chunks\"")?;
        let total_csize = stats
            .get("total_csize")
            .and_then(|t| t.as_u64())
            .ok_or("missing key: \"cache.stats.total_csize\"")?;
        let total_size = stats
            .get("total_size")
            .and_then(|t| t.as_u64())
            .ok_or("missing key: \"cache.stats.total_size\"")?;
        let total_unique_chunks = stats
            .get("total_unique_chunks")
            .and_then(|t| t.as_u64())
            .ok_or("missing key: \"cache.stats.total_unique_chunks\"")?;
        let unique_csize = stats
            .get("unique_csize")
            .and_then(|t| t.as_u64())
            .ok_or("missing key: \"cache.stats.unique_csize\"")?;
        let unique_size = stats
            .get("unique_size")
            .and_then(|t| t.as_u64())
            .ok_or("missing key: \"cache.stats.unique_size\"")?;
        let encryption = value
            .get("encryption")
            .and_then(|e| e.as_object())
            .ok_or("missing key: \"encryption\"")?;

        let encryption = match encryption
            .get("mode")
            .and_then(|m| m.as_str())
            .ok_or("missing key: \"encryption.mode\"")?
        {
            "none" => Encryption::None,
            "repokey" => Encryption::RepoKey,
            "repokey-blake2" => Encryption::RepoKeyBlake2,
            "keyfile" => Encryption::KeyFile,
            "keyfile-blake2" => Encryption::KeyFileBlake2,
            "authenticated" => Encryption::Authenticated,
            "authenticated-blake2" => Encryption::AuthenticatedBlake2,
            _ => return Err("unsupported encryption mode".into()),
        };

        let id = value
            .get("repository")
            .and_then(|r| r.get("id"))
            .and_then(|i| i.as_str())
            .ok_or("missing key: \"repository.id\"")?
            .to_owned();
        let location = value
            .get("repository")
            .and_then(|r| r.get("location"))
            .and_then(|l| l.as_str())
            .ok_or("missing key: \"repository.location\"")?
            .to_owned();
        let security_dir = value
            .get("security_dir")
            .and_then(|s| s.as_str())
            .map(|s| PathBuf::from(s))
            .ok_or("missing key: \"security_dir\"")?;

        Ok(RepoInfo {
            cache_path,
            total_chunks,
            total_csize,
            total_size,
            total_unique_chunks,
            unique_csize,
            unique_size,
            encryption,
            id,
            location,
            security_dir,
        })
    }
}

#[derive(Debug)]
pub struct BorgWrapper {
    path: PathBuf,
}

impl BorgWrapper {
    pub fn from_path(path: PathBuf) -> Self {
        Self { path }
    }

    fn build_command(&self) -> Command {
        Command::new(&self.path)
    }

    fn pass_rate_limit(&self, cmd: &mut Command, rate_limit: &RateLimit) {
        match rate_limit {
            RateLimit {
                up: Some(up),
                down: Some(down),
            } => {
                cmd.arg("--upload-ratelimit");
                cmd.arg(up.to_string());
                cmd.arg("--download-ratelimit");
                cmd.arg(down.to_string());
            }
            RateLimit {
                up: Some(up),
                down: None,
            } => {
                cmd.arg("--upload-ratelimit");
                cmd.arg(up.to_string());
            }
            RateLimit {
                up: None,
                down: Some(down),
            } => {
                cmd.arg("--download-ratelimit");
                cmd.arg(down.to_string());
            }
            _ => {}
        }
    }

    fn pass_passphrase(&self, cmd: &mut Command, passphrase: &Passphrase) {
        match passphrase {
            Passphrase::Passphrase(ref passphrase) => {
                cmd.env("BORG_PASSPHRASE", passphrase);
            }
            Passphrase::Command(ref command) => {
                cmd.env("BORG_PASSCOMMAND", command);
            }
            Passphrase::FileDescriptor(fd) => {
                cmd.env("BORG_PASSPHRASE_FD", fd.to_string());
            }
        }
    }
}

impl Default for BorgWrapper {
    fn default() -> Self {
        Self::from_path("borg".into())
    }
}

impl Backend for BorgWrapper {
    type Events = Events<ChildStderr>;

    fn init_repository(
        &self,
        borg: &Borg<Self>,
        repository: &Repo,
        encryption: Encryption,
        append_only: bool,
    ) -> Result<Repo> {
        todo!()
    }

    fn create_archive(
        &self,
        borg: &Borg<Self>,
        repository: &Repo,
        archive: &Archive,
    ) -> Result<Self::Events> {
        if archive.paths.is_empty() {
            return Err("No paths specified".into());
        }

        let mut cmd = self.build_command();

        self.pass_rate_limit(&mut cmd, &borg.rate_limit);

        if let Some(pass) = &repository.passphrase {
            self.pass_passphrase(&mut cmd, &pass);
        }

        cmd.arg("create");

        // TODO: make this configurable
        cmd.arg("--progress");
        cmd.arg("--stats");
        // cmd.arg("--list");
        cmd.arg("--log-json");

        if borg.dry_run {
            cmd.arg("--dry-run");
        }

        if let Some(comment) = &archive.comment {
            cmd.arg("--comment").arg(comment);
        }

        if let Some(compression) = &archive.compression {
            cmd.arg("--compression").arg(compression.to_string());
        }

        if let Some(pattern_file) = &archive.pattern_file {
            let pattern_file = if pattern_file.is_absolute() {
                pattern_file.to_owned()
            } else if let Some(path) = archive.paths.first() {
                path.join(pattern_file)
            } else {
                return Err("relative pattern file for multiple paths".into());
            };
            cmd.arg("--patterns-from");
            cmd.arg(pattern_file.to_string_lossy().to_string());
        }

        if let Some(exclude_file) = &archive.exclude_file {
            let exclude_file = if exclude_file.is_absolute() {
                exclude_file.to_owned()
            } else if let Some(path) = archive.paths.first() {
                path.join(exclude_file)
            } else {
                return Err("relative exclude file for multiple paths".into());
            };
            cmd.arg("--exclude-from");
            cmd.arg(exclude_file.to_string_lossy().to_string());
        }

        cmd.arg(format!("{}::{}", repository.location, archive.name));
        cmd.args(
            archive
                .paths
                .iter()
                .map(|p| p.to_string_lossy().to_string()),
        );

        log_command(&cmd);

        cmd.stderr(Stdio::piped());
        let mut child = cmd.spawn()?;

        let stderr = child.stderr.take();

        let stderr = match stderr {
            Some(stderr) => stderr,
            None => return Err("No stderr".into()),
        };

        Ok(stderr.into())
    }

    fn repo_info(&self, repository: &Repo) -> Result<RepoInfo> {
        let mut cmd = self.build_command();

        cmd.arg("info");

        if let Some(pass) = &repository.passphrase {
            self.pass_passphrase(&mut cmd, &pass);
        }

        cmd.arg("--json");
        cmd.arg(&repository.location);

        log_command(&cmd);

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into());
        }

        let json = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;

        Ok(json.try_into()?)
    }
}
