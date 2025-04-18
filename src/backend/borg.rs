use crate::{
    borg::{log::Event, Archive, Passphrase, Repo},
    borrg::*,
    util::ByteSize,
};
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
    process::{Command, Stdio},
};

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

        Ok(Self {
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
        }
        self
    }

    pub(self) fn passphrase(&mut self, passphrase: &Passphrase) -> &mut Self {
        match *passphrase {
            Passphrase::None => {}
            Passphrase::Phrase(ref passphrase) => {
                self.env("BORG_PASSPHRASE", passphrase);
            }
            Passphrase::Command(ref command) => {
                self.env("BORG_PASSCOMMAND", command);
            }
            Passphrase::Fd(fd) => {
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
        repo: &Repo,
        passphrase: &Passphrase,
        encryption: Encryption,
        append_only: bool,
        storage_quota: Option<ByteSize>,
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

        cmd.arg(repo.to_string());

        cmd.passphrase(passphrase);

        // Don't let borg ask if the passphrase should be displayed
        cmd.env("BORG_DISPLAY_PASSPHRASE", "no");

        dbg!(&cmd.0);

        cmd.stderr(Stdio::piped());
        let mut child = cmd.spawn()?;

        let stderr = child.stderr.take();

        let stderr = match stderr {
            Some(stderr) => stderr,
            None => return Err("No stderr".into()),
        };

        for event in crate::borg::log::read(stderr) {
            on_update(event);
        }

        Ok(())
    }

    fn create_archive(borg: &Borg, archive: &Archive, on_update: impl Fn(Event)) -> Result<()> {
        let mut cmd = BorgCommand::default();

        cmd.rate_limit(&borg.rate_limit);

        cmd.arg("create");

        // TODO: make this configurable
        cmd.progress();
        // cmd.arg("--list");
        cmd.arg("--log-json");

        if borg.dry_run {
            cmd.arg("--dry-run");
        } else {
            cmd.arg("--stats");
        }

        archive.apply_args(&mut cmd)?;

        dbg!(&cmd.0);

        cmd.stderr(Stdio::piped());
        let mut child = cmd.spawn()?;

        let stderr = child.stderr.take();

        let stderr = match stderr {
            Some(stderr) => stderr,
            None => return Err("No stderr".into()),
        };

        for event in crate::borg::log::read(stderr) {
            on_update(event);
        }

        Ok(())
    }

    fn repo_info(repository: &Repo, passphrase: &Passphrase) -> Result<RepoInfo> {
        let mut cmd = BorgCommand::default();

        cmd.arg("info");

        cmd.passphrase(passphrase);

        cmd.arg("--json");
        cmd.arg(repository.to_string());

        dbg!(&cmd.0);

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into());
        }

        let json = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;

        json.try_into()
    }
}
