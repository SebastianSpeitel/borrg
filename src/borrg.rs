use std::fmt::Display;
use std::path::PathBuf;

use crate::{
    borg::{Archive, Passphrase, Repo},
    util::ByteSize,
};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
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
        storage_quota: Option<ByteSize>,
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
        storage_quota: Option<ByteSize>,
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
