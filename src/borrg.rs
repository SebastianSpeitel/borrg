use std::fmt::Display;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::borg::{Archive, Passphrase, Repo};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, clap::ValueEnum)]
#[non_exhaustive]
pub enum Encryption {
    None,
    RepoKey,
    RepoKeyBlake2,
    KeyFile,
    KeyFileBlake2,
    Authenticated,
    AuthenticatedBlake2,
}

impl Display for Encryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::RepoKey => write!(f, "repokey"),
            Self::RepoKeyBlake2 => write!(f, "repokey-blake2"),
            Self::KeyFile => write!(f, "keyfile"),
            Self::KeyFileBlake2 => write!(f, "keyfile-blake2"),
            Self::Authenticated => write!(f, "authenticated"),
            Self::AuthenticatedBlake2 => write!(f, "authenticated-blake2"),
        }
    }
}

#[derive(Debug)]
pub enum Event {
    ArchiveProgress {
        nfiles: u64,
        compressed_size: u64,
        deduplicated_size: u64,
        original_size: u64,
        path: PathBuf,
        time: Option<SystemTime>,
    },
    ProgressMessage {
        message: Option<String>,
        finished: Option<bool>,
        msgid: Option<String>,
        operation: Option<u64>,
        time: Option<SystemTime>,
    },
    ProgressPercent {
        current: u64,
        finished: bool,
        message: String,
        msgid: String,
        operation: u64,
        time: SystemTime,
        total: u64,
    },
    LogMessage {
        name: Option<String>,
        level: Option<log::Level>,
        message: String,
        msgid: Option<String>,
        time: Option<SystemTime>,
    },
    FileStatus {
        status: String,
        path: PathBuf,
    },
    Prompt {
        prompt: String,
        msgid: String,
    },
    Answer {
        answer: String,
        env_var: Option<String>,
        msgid: String,
    },
    Other(String),
    Error(Error),
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Event::*;
        if f.alternate() {
            return <Self as std::fmt::Debug>::fmt(self, f);
        }
        match self {
            ArchiveProgress {
                nfiles,
                compressed_size,
                deduplicated_size,
                original_size,
                path,
                ..
            } => {
                // 3.40 GB O 2.07 GB C 0 B D 8423 N [path]
                write!(
                    f,
                    "{} O {} C {} D {nfiles} N {}",
                    ByteSize(*original_size),
                    ByteSize(*compressed_size),
                    ByteSize(*deduplicated_size),
                    path.display()
                )
            }
            ProgressMessage { message, .. } => {
                if let Some(message) = message {
                    write!(f, "{message}")
                } else {
                    Ok(())
                }
            }
            LogMessage { message, .. } => {
                write!(f, "{message}")
            }
            ProgressPercent { message, .. } => write!(f, "{message}"),
            FileStatus { path, status } => write!(f, "{} {}", status, path.display()),
            Prompt { prompt, .. } => write!(f, "{prompt}"),
            Answer { answer, .. } => write!(f, "{answer}"),
            Other(s) => write!(f, "{s}"),
            Error(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Default, Debug)]
pub struct RateLimit {
    pub up: Option<u64>,
    pub down: Option<u64>,
}

#[derive(Debug)]
pub struct RepoInfo {
    pub cache_path: PathBuf,
    pub total_chunks: u64,
    pub total_csize: u64,
    pub total_size: u64,
    pub total_unique_chunks: u64,
    pub unique_csize: u64,
    pub unique_size: u64,
    pub encryption: Encryption,
    pub id: String,
    pub location: String,
    // pub(crate) last_modified: SystemTime,
    pub security_dir: PathBuf,
    // "cache": {
    //     "path": "/home/seb/.cache/borg/dd06d1d72e5925b63f9c929b088b1cfa2e6bd548f5037c05352a61d71e4d2819",
    //     "stats": {
    //         "total_chunks": 236619767,
    //         "total_csize": 26289835627221,
    //         "total_size": 38449962381221,
    //         "total_unique_chunks": 1621026,
    //         "unique_csize": 300958014008,
    //         "unique_size": 477242905022
    //     }
    // },
    // "encryption": {
    //     "mode": "repokey"
    // },
    // "repository": {
    //     "id": "dd06d1d72e5925b63f9c929b088b1cfa2e6bd548f5037c05352a61d71e4d2819",
    //     "last_modified": "2022-04-07T15:44:37.000000",
    //     "location": "ssh://borg.backup/~/sagittarius"
    // },
    // "security_dir": "/home/seb/.config/borg/security/dd06d1d72e5925b63f9c929b088b1cfa2e6bd548f5037c05352a61d71e4d2819"
}

#[derive(Debug, Default)]
pub struct Borg {
    pub(crate) dry_run: bool,
    pub(crate) rate_limit: RateLimit,
}

impl Borg {
    pub fn dry_run(&mut self) -> &mut Self {
        self.dry_run = true;
        self
    }

    pub fn init_repository<B: Backend>(
        &self,
        repo: &Repo,
        passphrase: &Passphrase,
        encryption: Encryption,
        append_only: bool,
        storage_quota: Option<usize>,
        make_parent_dirs: bool,
        on_update: impl Fn(B::Update),
    ) -> Result<()> {
        B::init_repository(
            self,
            repo,
            passphrase,
            encryption,
            append_only,
            storage_quota,
            make_parent_dirs,
            on_update,
        )
    }

    pub fn create_archive<B: Backend>(
        &self,
        archive: &Archive,
        on_update: impl Fn(B::Update),
    ) -> Result<()> {
        B::create_archive(self, archive, on_update)
    }
}

pub trait Backend {
    type Update: Display;

    /// Initialize an empty repository
    fn init_repository(
        borg: &Borg,
        repo: &Repo,
        passphrase: &Passphrase,
        encryption: Encryption,
        append_only: bool,
        storage_quota: Option<usize>,
        make_parent_dirs: bool,
        on_update: impl Fn(Self::Update),
    ) -> Result<()>;

    /// Create new archive
    fn create_archive(
        borg: &Borg,
        archive: &Archive,
        on_update: impl Fn(Self::Update),
    ) -> Result<()>;

    fn repo_info(repository: &Repo, passphrase: &Passphrase) -> Result<RepoInfo>;
}

pub struct ByteSize(pub u64);

impl ByteSize {
    const SUFFIX_SI: [&'static str; 9] = ["", "K", "M", "G", "T", "P", "E", "Z", "Y"];
    const SUFFIX_IEC: [&'static str; 9] = ["", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi"];

    #[inline]
    pub fn iec(&self, precision: Option<usize>) -> String {
        let bytes = self.0 as f64;
        if bytes < 1024.0 {
            return bytes.to_string();
        }
        let base = (bytes.log2() / 10_f64) as usize;
        assert!(base < 9);
        format!(
            "{:.*}{}",
            precision.unwrap_or(0),
            bytes / 1024.0f64.powi(base as i32),
            Self::SUFFIX_IEC[base]
        )
    }

    #[inline]
    pub fn si(&self, precision: Option<usize>) -> String {
        let bytes = self.0 as f64;
        if bytes < 1000_f64 {
            return bytes.to_string();
        }
        let base = (bytes.log10() / 3_f64) as usize;
        assert!(base < 9);
        format!(
            "{:.*}{}",
            precision.unwrap_or(0),
            bytes / 1000.0f64.powi(base as i32),
            Self::SUFFIX_SI[base]
        )
    }
}

impl std::fmt::Display for ByteSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match f.alternate() {
            false => f.write_str(&self.iec(f.precision())),
            true => f.write_str(&self.si(f.precision())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_size() {
        assert_eq!(ByteSize(0).iec(None), "0");
        assert_eq!(ByteSize(0).si(None), "0");
        assert_eq!(ByteSize(1).iec(None), "1");
        assert_eq!(ByteSize(1).si(None), "1");
        assert_eq!(ByteSize(10).iec(None), "10");
        assert_eq!(ByteSize(10).si(None), "10");
        assert_eq!(ByteSize(100).iec(None), "100");
        assert_eq!(ByteSize(100).si(None), "100");
        assert_eq!(ByteSize(1000).iec(None), "1000");
        assert_eq!(ByteSize(1000).si(None), "1K");
        assert_eq!(ByteSize(1024).iec(None), "1Ki");
        assert_eq!(ByteSize(1024).si(None), "1K");
        assert_eq!(ByteSize(1024).iec(Some(3)), "1.000Ki");
        assert_eq!(ByteSize(1024).si(Some(3)), "1.024K");
        assert_eq!(ByteSize(1025).iec(None), "1Ki");
        assert_eq!(ByteSize(1025).si(None), "1K");
        assert_eq!(ByteSize(1025).iec(Some(0)), "1Ki");
        assert_eq!(ByteSize(1025).si(Some(0)), "1K");
        assert_eq!(ByteSize(1025).iec(Some(1)), "1.0Ki");
        assert_eq!(ByteSize(1025).si(Some(1)), "1.0K");
        assert_eq!(ByteSize(1025).iec(Some(2)), "1.00Ki");
        assert_eq!(ByteSize(1025).si(Some(2)), "1.02K");
        assert_eq!(ByteSize(1025).iec(Some(3)), "1.001Ki");
        assert_eq!(ByteSize(1025).si(Some(3)), "1.025K");
    }
}
