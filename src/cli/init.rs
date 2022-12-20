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
    let backup = config.backups.iter().map(|(r, _)| r).find(|r| r == &&repo);

    let mut exists_already = false;
    if let Some(backup) = backup {
        repo.passphrase = backup.passphrase.clone();
        exists_already = true;
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

    if !exists_already {
        if let Err(e) = append_backup_config(&config.source, &repo) {
            eprintln!("Failed to append backup to config: {}", e);
        }
    }
}

fn append_backup_config(
    path: &std::path::PathBuf,
    repo: &crate::Repo,
) -> Result<(), std::io::Error> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new().append(true).open(path)?;

    file.write(b"\n[[backup]]\nrepository = \"")?;
    file.write(repo.to_string().as_bytes())?;
    file.write(b"\"\n")?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_init() {
        let args = super::Args {
            encryption: Encryption::None,
            append_only: false,
            storage_quota: None,
            make_parent_dirs: false,
            repository: "/tmp/test-repo".parse().unwrap(),
        };

        let config_path = std::path::PathBuf::from("/tmp/borrg.toml");

        // Cleanup
        std::fs::remove_file(&config_path).ok();
        std::fs::remove_dir_all("/tmp/test-repo").ok();

        std::fs::write(&config_path, "").unwrap();

        let borg = Borg::default();
        let config = Config::load(&config_path).unwrap();

        init(borg, config, args);

        let config_after = Config::load(&config_path).unwrap();
        assert_eq!(config_after.backups.len(), 1);
    }
}
