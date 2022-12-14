use std::fmt::Display;
use std::num::NonZeroU8;
use std::path::PathBuf;
use std::time::SystemTime;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Passphrase {
    Passphrase(String),
    Command(String),
    FileDescriptor(i32),
}

#[derive(Debug)]
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

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Compression {
    None {
        obfuscation: Option<NonZeroU8>,
    },
    Lz4 {
        auto: bool,
        obfuscation: Option<NonZeroU8>,
    },
    Zstd {
        level: Option<u8>,
        auto: bool,
        obfuscation: Option<NonZeroU8>,
    },
    Zlib {
        level: Option<u8>,
        auto: bool,
        obfuscation: Option<NonZeroU8>,
    },
    Lzma {
        level: Option<u8>,
        auto: bool,
        obfuscation: Option<NonZeroU8>,
    },
}

impl Display for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_obfuscation(obfuscate: &Option<NonZeroU8>) -> String {
            obfuscate.map_or("".to_string(), |o| format!("obfuscate,{},", o.get()))
        }

        fn fmt_auto(auto: &bool) -> String {
            if *auto {
                "auto,".to_string()
            } else {
                "".to_string()
            }
        }

        fn fmt_level(level: &Option<u8>) -> String {
            level.map_or("".to_string(), |l| format!(",{}", l))
        }

        use Compression::*;
        match self {
            None { obfuscation } => {
                write!(f, "{}none", fmt_obfuscation(obfuscation))
            }
            Lz4 { auto, obfuscation } => {
                write!(f, "{}lz4{}", fmt_obfuscation(obfuscation), fmt_auto(auto))
            }
            Zstd {
                level,
                auto,
                obfuscation,
            } => {
                write!(
                    f,
                    "{}{}zstd{}",
                    fmt_obfuscation(obfuscation),
                    fmt_auto(auto),
                    fmt_level(level)
                )
            }
            Zlib {
                level,
                auto,
                obfuscation,
            } => {
                write!(
                    f,
                    "{}{}zlib{}",
                    fmt_obfuscation(obfuscation),
                    fmt_auto(auto),
                    fmt_level(level)
                )
            }
            Lzma {
                level,
                auto,
                obfuscation,
            } => {
                write!(
                    f,
                    "{}{}lzma{}",
                    fmt_obfuscation(obfuscation),
                    fmt_auto(auto),
                    fmt_level(level)
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct Archive {
    pub(crate) name: String,
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) compression: Option<Compression>,
    pub(crate) pattern_file: Option<PathBuf>,
    pub(crate) exclude_file: Option<PathBuf>,
    pub(crate) comment: Option<String>,
}

impl Archive {
    pub fn new(name: String) -> Self {
        Archive {
            name,
            paths: Vec::new(),
            compression: None,
            pattern_file: None,
            exclude_file: None,
            comment: None,
        }
    }

    #[cfg(feature = "chrono")]
    pub fn today() -> Self {
        let now = chrono::Local::now();
        let name = now.format("%Y-%m-%d").to_string();
        Archive::new(name)
    }

    pub fn path(&mut self, path: PathBuf) -> &mut Self {
        self.paths.push(path);
        self
    }

    pub fn compression(&mut self, compression: Compression) -> &mut Self {
        self.compression.replace(compression);
        self
    }

    pub fn pattern_file(&mut self, pattern_file: PathBuf) -> &mut Self {
        self.pattern_file.replace(pattern_file);
        self
    }

    pub fn exclude_file(&mut self, exclude_file: PathBuf) -> &mut Self {
        self.exclude_file.replace(exclude_file);
        self
    }

    pub fn comment(&mut self, comment: String) -> &mut Self {
        self.comment.replace(comment);
        self
    }
}

impl Display for Archive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
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
                    write!(f, "{}", message)
                } else {
                    Ok(())
                }
            }
            LogMessage { message, .. } => {
                write!(f, "{}", message)
            }
            ProgressPercent { message, .. } => write!(f, "{message}"),
            FileStatus { path, status } => write!(f, "{} {}", status, path.display()),
            Prompt { prompt, .. } => write!(f, "{}", prompt),
            Other(s) => write!(f, "{}", s),
            Error(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Debug)]
pub struct Repo {
    pub(crate) location: String,
    pub(crate) passphrase: Option<Passphrase>,
}

impl Repo {
    pub fn new(location: String) -> Self {
        Self {
            location,
            passphrase: None,
        }
    }

    pub fn passphrase(&mut self, passphrase: Passphrase) -> &mut Self {
        self.passphrase = Some(passphrase);
        self
    }

    pub fn create_archive<B: Backend>(
        &self,
        borg: &Borg,
        archive: &Archive,
        on_update: impl Fn(B::Update),
    ) -> Result<()> {
        B::create_archive(borg, self, archive, on_update)
    }

    pub fn info<B: Backend>(&self) -> Result<RepoInfo> {
        B::repo_info(self)
    }
}

impl Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.location)
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

    pub fn init<B: Backend>(
        &self,
        repository: &Repo,
        encryption: Encryption,
        append_only: bool,
    ) -> Result<Repo> {
        B::init_repository(self, repository, encryption, append_only)
    }

    pub fn create_archive<B: Backend>(
        &self,
        repository: &Repo,
        archive: &Archive,
        on_update: impl Fn(B::Update),
    ) -> Result<()> {
        B::create_archive(self, repository, archive, on_update)
    }
}

pub trait Backend {
    type Update: Display;

    /// Initialize an empty repository
    fn init_repository(
        borg: &Borg,
        repository: &Repo,
        encryption: Encryption,
        append_only: bool,
    ) -> Result<Repo>;

    /// Create new archive
    fn create_archive(
        borg: &Borg,
        repository: &Repo,
        archive: &Archive,
        on_update: impl Fn(Self::Update),
    ) -> Result<()>;

    fn repo_info(repository: &Repo) -> Result<RepoInfo>;
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
