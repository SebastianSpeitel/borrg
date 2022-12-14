use super::*;
use crate::{backend, Borg, Encryption};

#[derive(Args, Debug)]
pub struct Args {
    /// Select encryption key mode
    #[arg(short, long, value_enum)]
    encryption: Encryption,

    /// Create an append-only mode repository. Note that this only affects the low level structure of the repository, and running `delete` or `prune` will still be allowed.
    #[arg(long)]
    append_only: bool,

    /// Set storage quota of the new repository (e.g. 5G, 1.5T). Default: no quota.
    #[arg(long,value_parser = parse_byte_size)]
    storage_quota: Option<usize>,

    /// Create the parent directories of the repository directory, if they are missing.
    #[arg(long, default_value = "false")]
    make_parent_dirs: bool,

    /// Path to the new repository
    #[arg(value_name = "REPOSITORY")]
    repository: crate::Repo,
}

pub fn init(borg: Borg, config: Config, args: Args) {
    let mut repo = args.repository;

    // Search matching backup in config
    let backup = config
        .backups
        .iter()
        .map(|(b, _)| b)
        .find(|b| b.location == repo.location);

    if let Some(backup) = backup {
        repo.passphrase = backup.passphrase.clone();
    }

    if let Err(e) = borg.init_repository::<backend::borg::BorgWrapper>(
        &mut repo,
        args.encryption,
        args.append_only,
        args.storage_quota,
        args.make_parent_dirs,
        |u| {
            println!("{}", u);
        },
    ) {
        eprintln!("Failed to initialize repository: {}", e);
        std::process::exit(1);
    }
}
