use borrg::Borg;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
mod util;

/// Borrg wrapper
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// Path to config file
    #[clap(short, long, default_value = "~/.config/borg/borrg.toml")]
    config: PathBuf,

    /// Run borg in dry run mode
    #[clap(long)]
    dry_run: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run all configured backups
    Run(borrg::cli::run::Args),
    /// Initialize a new borg repository
    Init(borrg::cli::init::Args),
    /// List backups
    List,
    /// Get info about a backup
    Info { backup: String },
    /// Validate config
    Debug,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    let config_path = util::resolve_path(&cli.config);
    let config = borrg::cli::Config::load(&config_path);

    let config = match config {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config ({}): {}", config_path.display(), e);
            std::process::exit(1);
        }
    };

    let mut borg = Borg::default();
    if cli.dry_run {
        borg.dry_run();
    }

    match cli.command {
        Commands::Debug => {
            dbg!(cli);
            dbg!(config);
            dbg!(borg);
        }
        Commands::Run(args) => {
            borrg::cli::run::run(borg, config, args);
        }
        Commands::Init(args) => {
            borrg::cli::init::init(borg, config, args);
        }
        _ => unimplemented!(),
    }

    Ok(())
}
