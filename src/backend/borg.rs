use crate::{borrg::*, util::resolve_path};
use log::{debug, trace, warn, Level};
use std::{
    io::{BufRead, BufReader, Lines, Read},
    ops::{Deref, DerefMut},
    path::PathBuf,
    process::{ChildStderr, Command, Stdio},
    time::{Duration, SystemTime},
};

impl TryFrom<serde_json::Value> for Event {
    type Error = Error;
    fn try_from(value: serde_json::Value) -> Result<Self> {
        let _type = match value.get("type") {
            Some(serde_json::Value::String(t)) => t,
            _ => return Err("no type".into()),
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
                .map(PathBuf::from)
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
            if let Some(l) = value
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
            {
                return Some(l);
            }

            if let Some(l) = value
                .get("levelname")
                .and_then(|l| l.as_str())
                .and_then(|l| match l {
                    "DEBUG" => Some(Level::Debug),
                    "INFO" => Some(Level::Info),
                    "WARNING" => Some(Level::Warn),
                    "ERROR" => Some(Level::Error),
                    _ => {
                        warn!("unknown log level: {}", l);
                        None
                    }
                })
            {
                return Some(l);
            }

            None
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
        let current = || value.get("current").and_then(|c| c.as_u64());
        let total = || value.get("total").and_then(|t| t.as_u64());

        let event = match _type.as_str() {
            "archive_progress" => Self::ArchiveProgress {
                nfiles: nfiles().unwrap_or_default(),
                compressed_size: compressed_size().unwrap_or_default(),
                deduplicated_size: deduplicated_size().unwrap_or_default(),
                original_size: original_size().unwrap_or_default(),
                path: path().unwrap_or_default(),
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
                message: message().unwrap_or_default(),
                msgid: msgid(),
                time: time(),
            },
            "file_status" => Self::FileStatus {
                path: path().unwrap_or_default(),
                status: status().unwrap_or_default(),
            },
            "progress_percent" => Self::ProgressPercent {
                current: current().unwrap_or_default(),
                finished: finished().unwrap_or_default(),
                message: message().unwrap_or_default(),
                msgid: msgid().unwrap_or_default(),
                operation: operation().unwrap_or_default(),
                time: time().unwrap_or_else(|| {
                    warn!("no time in progress_percent");
                    SystemTime::now()
                }),
                total: total().unwrap_or_default(),
            },
            "question_prompt" => Self::Prompt {
                prompt: message().unwrap(),
                msgid: msgid().unwrap(),
            },
            _ => return Err(format!("Unknown event type: {}", _type).into()),
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

        trace!("[borg] {:#?}", line);

        let json: std::result::Result<serde_json::Value, _> = serde_json::from_str(&line);
        let json = match json {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to parse borg log event: {line:?} ({e})");
                return Some(Event::Other(line));
            }
        };

        debug!("{:#?}", json);

        match Event::try_from(json) {
            Ok(event) => {
                debug!("{:#?}", event);
                Some(event)
            }
            Err(e) => {
                warn!("Unknown borg log event: {line:?} ({e})");
                Some(Event::Other(line))
            }
        }
    }
}

fn log_command(cmd: &Command) {
    let command = format!(
        "{} {}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args()
            .map(|a| format!("\"{}\"", a.to_string_lossy()))
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
            .map(PathBuf::from)
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
            .map(PathBuf::from)
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

struct BorgCommand(Command);

impl BorgCommand {
    pub(self) fn rate_limit(&mut self, rate_limit: &RateLimit) -> &mut Self {
        match rate_limit {
            RateLimit {
                up: Some(up),
                down: Some(down),
            } => {
                self.arg("--upload-ratelimit");
                self.arg(up.to_string());
                self.arg("--download-ratelimit");
                self.arg(down.to_string());
            }
            RateLimit {
                up: Some(up),
                down: None,
            } => {
                self.arg("--upload-ratelimit");
                self.arg(up.to_string());
            }
            RateLimit {
                up: None,
                down: Some(down),
            } => {
                self.arg("--download-ratelimit");
                self.arg(down.to_string());
            }
            _ => {}
        };
        self
    }

    pub(self) fn passphrase(&mut self, passphrase: &Passphrase) -> &mut Self {
        match passphrase {
            Passphrase::Passphrase(ref passphrase) => {
                self.env("BORG_PASSPHRASE", passphrase);
            }
            Passphrase::Command(ref command) => {
                self.env("BORG_PASSCOMMAND", command);
            }
            Passphrase::FileDescriptor(fd) => {
                self.env("BORG_PASSPHRASE_FD", fd.to_string());
            }
        }
        self
    }

    pub(self) fn progress(&mut self) -> &mut Self {
        self.arg("--progress");
        self
    }

    pub(self) fn log_level(&mut self, level: log::Level) -> &mut Self {
        match level {
            log::Level::Error => self.arg("--error"),
            log::Level::Warn => self.arg("--warning"),
            log::Level::Info => self.arg("--info"),
            log::Level::Debug => self.arg("--debug"),
            log::Level::Trace => self.arg("--debug"),
        };
        self
    }
}

impl Default for BorgCommand {
    fn default() -> Self {
        let borg_path = std::env::var("BORG_PATH").unwrap_or_else(|_| "borg".to_owned());

        let mut cmd = Self(Command::new(borg_path));

        if let Some(level) = log::max_level().to_level() {
            cmd.log_level(level);
        };

        cmd
    }
}

impl Deref for BorgCommand {
    type Target = Command;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BorgCommand {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct BorgWrapper {}

impl Backend for BorgWrapper {
    type Update = Event;

    fn init_repository(
        borg: &Borg,
        repository: &mut Repo,
        encryption: Encryption,
        append_only: bool,
        storage_quota: Option<usize>,
        make_parent_dirs: bool,
        on_update: impl Fn(Event),
    ) -> Result<()> {
        let mut cmd = BorgCommand::default();

        cmd.arg("init");

        cmd.arg("--log-json");

        cmd.rate_limit(&borg.rate_limit);

        if append_only {
            cmd.arg("--append-only");
        }

        if make_parent_dirs {
            cmd.arg("--make-parent-dirs");
        }

        if let Some(quota) = storage_quota {
            cmd.arg("--storage-quota");
            cmd.arg(quota.to_string());
        }

        cmd.arg("--encryption");
        cmd.arg(encryption.to_string());

        cmd.arg(repository.to_string());

        if let Some(ref pass) = repository.passphrase {
            cmd.passphrase(pass);
        }

        // Don't let borg ask if the passphrase should be displayed
        cmd.env("BORG_DISPLAY_PASSPHRASE", "no");

        log_command(&cmd);

        cmd.stderr(Stdio::piped());
        let mut child = cmd.spawn()?;

        let stderr = child.stderr.take();

        let stderr = match stderr {
            Some(stderr) => stderr,
            None => return Err("No stderr".into()),
        };

        for event in Events::from(stderr) {
            on_update(event);
        }

        Ok(())
    }

    fn create_archive(
        borg: &Borg,
        repository: &Repo,
        archive: &Archive,
        on_update: impl Fn(Event),
    ) -> Result<()> {
        if archive.paths.is_empty() {
            return Err("No paths specified".into());
        }

        let mut cmd = BorgCommand::default();

        cmd.rate_limit(&borg.rate_limit);

        if let Some(pass) = &repository.passphrase {
            cmd.passphrase(pass);
        }

        cmd.arg("create");

        // TODO: make this configurable
        cmd.progress();
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
                resolve_path(&path.join(pattern_file))
            } else {
                return Err("relative pattern file for multiple paths".into());
            };
            if !pattern_file.is_file() {
                return Err(
                    format!("pattern file does not exist: {}", pattern_file.display()).into(),
                );
            }
            cmd.arg("--patterns-from");
            cmd.arg(pattern_file);
        }

        if let Some(exclude_file) = &archive.exclude_file {
            let exclude_file = if exclude_file.is_absolute() {
                exclude_file.to_owned()
            } else if let Some(path) = archive.paths.first() {
                resolve_path(&path.join(exclude_file))
            } else {
                return Err("relative exclude file for multiple paths".into());
            };
            if !exclude_file.is_file() {
                return Err(
                    format!("exclude file does not exist: {}", exclude_file.display()).into(),
                );
            }
            cmd.arg("--exclude-from");
            cmd.arg(exclude_file);
        }

        cmd.arg(format!("{}::{}", repository.location, archive.name));
        cmd.args(archive.paths.iter().map(resolve_path));

        log_command(&cmd);

        cmd.stderr(Stdio::piped());
        let mut child = cmd.spawn()?;

        let stderr = child.stderr.take();

        let stderr = match stderr {
            Some(stderr) => stderr,
            None => return Err("No stderr".into()),
        };

        for event in Events::from(stderr) {
            on_update(event);
        }

        Ok(())
    }

    fn repo_info(repository: &Repo) -> Result<RepoInfo> {
        let mut cmd = BorgCommand::default();

        cmd.arg("info");

        if let Some(pass) = &repository.passphrase {
            cmd.passphrase(pass);
        }

        cmd.arg("--json");
        cmd.arg(&repository.location);

        log_command(&cmd);

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into());
        }

        let json = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;

        json.try_into()
    }
}
