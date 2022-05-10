# Borrg

[![Rust](https://github.com/SebastianSpeitel/borrg/actions/workflows/main.yml/badge.svg)](https://github.com/SebastianSpeitel/borrg/actions/workflows/main.yml)

A borg wrapper written in rust

## Installation

```bash
cargo install --git https://github.com/SebastianSpeitel/borrg
```

## Usage
    
```bash
borrg --help
```

## Configuration

`~/.config/borg/borrg.toml`

```toml
[default]
compression = { algorithm = "zstd", level = 19, auto = true }
progress = true
stats = true

# Name of the backup
[remote]
repository = "remote:/path/to/backup"
passcommand = "sh -c 'pass backup | head -n1'"

[local]
repository = "/path/to/backup"
passphrase = "..."
```
