use borrg::Borg;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

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
    Run(borrg::cli::RunArgs),
    /// List backups
    List,
    /// Get info about a backup
    Info { backup: String },
    /// Validate config
    Debug,
}

#[derive(Args, Debug)]
struct Create {
    backup: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    let config_path = borrg::cli::resolve_path(&cli.config);
    let config: toml::Value = toml::from_str(&std::fs::read_to_string(&config_path)?)?;
    let config = match borrg::cli::Config::try_from(config) {
        Ok(c) => c,
        Err(e) => {
            return Err(e);
        }
    };

    let mut borg: Borg = Borg::default();
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
            borrg::cli::run(borg, config, args);
        }
        _ => unimplemented!(),
    }

    Ok(())
}
