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
[template.default]
# Default values inherited by each backup
compression = { algorithm = "zstd", level = 19, auto = true }
# Also valid: compression = "zstd"
progress = true
stats = true

[[backup]]
repository = "remote:/path/to/backup"
passcommand = "sh -c 'pass backup | head -n1'"
path = "/path/to/backup" # Defaults to "~"

[[backup]]
repository = "/path/to/repo"
passphrase = "..."
compression = "none"
```
