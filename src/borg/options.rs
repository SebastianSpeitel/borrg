use smol_str::SmolStr;

use crate::util::PathResolveExt;

use super::{Compression, Passphrase};

#[derive(Clone, Default)]
pub struct Options(pub(super) Vec<Opt>);

impl Options {
    #[inline]
    #[must_use]
    pub fn has_root(&self) -> bool {
        self.0.iter().any(|opt| matches!(opt, Opt::Root(_)))
    }

    #[inline]
    pub fn root(&mut self, root: impl Into<SmolStr>) {
        self.0.push(Opt::Root(root.into()));
    }

    #[inline]
    pub fn exclude_from(&mut self, exclude: impl Into<SmolStr>) {
        self.0.push(Opt::ExcludeFrom(exclude.into()));
    }

    #[inline]
    pub(super) fn apply(&self, command: &mut std::process::Command) {
        for opt in &self.0 {
            match opt {
                Opt::Root(_) => {
                    // Supplied as argument
                }
                Opt::Exclude(excl) => {
                    command.arg("--exclude");
                    command.arg(excl);
                }
                Opt::ExcludeFrom(excl_from) => {
                    command.arg("--exclude-from");
                    command.arg(excl_from.resolve().as_ref());
                }
                Opt::Pattern(pattern) => {
                    command.arg("--pattern");
                    command.arg(pattern);
                }
                Opt::PatternFrom(pattern_from) => {
                    command.arg("--pattern-from");
                    command.arg(pattern_from.resolve().as_ref());
                }
                Opt::ExcludeCaches => {
                    command.arg("--exclude-caches");
                }
                Opt::ExcludeNoDump => {
                    command.arg("--exclude-nodump");
                }
                Opt::OneFileSystem => {
                    command.arg("--one-file-system");
                }
                Opt::Comment(comment) => {
                    command.arg("--comment");
                    command.arg(comment);
                }
                Opt::Compression(compression) => {
                    command.arg("--compression");
                    command.arg(compression.as_smol_str());
                }
                Opt::Passphrase(passphrase) => match *passphrase {
                    Passphrase::None => {}
                    Passphrase::Phrase(ref passphrase) => {
                        command.env("BORG_PASSPHRASE", passphrase);
                    }
                    Passphrase::Command(ref passcommand) => {
                        command.env("BORG_PASSCOMMAND", passcommand);
                    }
                    Passphrase::Fd(fd) => {
                        command.env("BORG_PASSPHRASE_FD", fd.to_string());
                    }
                },
                Opt::RemotePath(remote_path) => {
                    command.env("BORG_REMOTE_PATH", remote_path);
                }
            }
        }
    }
}

impl core::fmt::Debug for Options {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl core::ops::Add for Options {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let mut opts = self.0;
        opts.extend(rhs.0);
        Self(opts)
    }
}

#[derive(Debug, Clone)]
pub(super) enum Opt {
    Root(SmolStr),
    Exclude(SmolStr),
    ExcludeFrom(SmolStr),
    Pattern(SmolStr),
    PatternFrom(SmolStr),
    ExcludeCaches,
    ExcludeNoDump,
    OneFileSystem,
    Comment(SmolStr),
    Compression(Compression),
    Passphrase(Passphrase),
    RemotePath(SmolStr),
}

impl TryFrom<&toml::Table> for Options {
    type Error = &'static str;

    #[inline]
    fn try_from(tab: &toml::Table) -> Result<Self, Self::Error> {
        use toml::Value;
        let mut opts = Vec::with_capacity(tab.len());

        fn unpack(
            opts: &mut Vec<Opt>,
            val: &Value,
            f: impl Fn(SmolStr) -> Opt,
        ) -> Result<(), &'static str> {
            match val {
                Value::String(s) => {
                    opts.push(f(s.into()));
                }
                Value::Array(arr) => {
                    for v in arr {
                        opts.push(f(v.as_str().ok_or("expected string")?.into()));
                    }
                }
                _ => return Err("expected string or array of strings"),
            }
            Ok(())
        }

        for (k, v) in tab {
            match (k.as_str(), v) {
                ("root" | "path", val) => {
                    unpack(&mut opts, val, Opt::Root)?;
                }
                ("exclude", val) => {
                    unpack(&mut opts, val, Opt::Exclude)?;
                }
                ("exclude_from", val) => {
                    unpack(&mut opts, val, Opt::ExcludeFrom)?;
                }
                ("pattern", val) => {
                    unpack(&mut opts, val, Opt::Pattern)?;
                }
                ("pattern_from", val) => {
                    unpack(&mut opts, val, Opt::PatternFrom)?;
                }
                ("exclude_caches", Value::Boolean(true)) => {
                    opts.push(Opt::ExcludeCaches);
                }
                ("exclude_nodump", Value::Boolean(true)) => {
                    opts.push(Opt::ExcludeNoDump);
                }
                ("one_file_system", Value::Boolean(true)) => {
                    opts.push(Opt::OneFileSystem);
                }
                ("comment", Value::String(s)) => {
                    opts.push(Opt::Comment(s.into()));
                }
                ("compression", val) => {
                    opts.push(Opt::Compression(Compression::try_from(val)?));
                }
                ("passphrase", val) => {
                    opts.push(Opt::Passphrase(Passphrase::try_from(val)?));
                }
                ("passcommand", Value::String(s)) => {
                    opts.push(Opt::Passphrase(Passphrase::Command(s.into())));
                }
                ("remote_path", Value::String(s)) => {
                    opts.push(Opt::RemotePath(s.into()));
                }
                _ => {}
            }
        }

        Ok(Self(opts))
    }
}
