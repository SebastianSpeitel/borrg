use smol_str::SmolStr;

use crate::util::PathResolveExt;

use super::{Compression, Opt, Options, Passphrase, Repo};

#[derive(Debug, Clone)]
pub struct Archive {
    pub(crate) repo: Repo<'static>,
    pub(crate) name: SmolStr,
    pub(crate) options: Options,
}

impl Archive {
    #[inline]
    pub fn new(repo: Repo<'static>, name: impl AsRef<str>) -> Self {
        Self {
            repo,
            name: name.as_ref().into(),
            options: Options::default(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn name(&self) -> &SmolStr {
        &self.name
    }

    #[inline]
    pub fn roots(&self) -> impl Iterator<Item = &SmolStr> {
        self.options.0.iter().filter_map(|opt| match opt {
            Opt::Root(root) => Some(root),
            _ => None,
        })
    }

    #[inline]
    #[must_use]
    pub fn compression(&self) -> Option<Compression> {
        self.options.0.iter().rev().find_map(|opt| match *opt {
            Opt::Compression(c) => Some(c),
            _ => None,
        })
    }

    #[inline]
    #[must_use]
    pub fn passphrase(&self) -> Option<&Passphrase> {
        self.options.0.iter().rev().find_map(|opt| match *opt {
            Opt::Passphrase(ref p) => Some(p),
            _ => None,
        })
    }

    #[inline]
    pub fn apply_args(&self, command: &mut std::process::Command) -> Result<(), &'static str> {
        let mut roots = Vec::new();

        for opt in &self.options.0 {
            match opt {
                Opt::Root(r) => {
                    roots.push(r);
                }
                Opt::Exclude(excl) => {
                    command.arg("--exclude");
                    command.arg(excl);
                }
                Opt::ExcludeFrom(excl_from) => {
                    command.arg("--exclude-from");
                    command.arg(excl_from);
                }
                Opt::Pattern(pattern) => {
                    command.arg("--pattern");
                    command.arg(pattern);
                }
                Opt::PatternFrom(pattern_from) => {
                    command.arg("--pattern-from");
                    command.arg(pattern_from);
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
            }
        }

        command.env("BORG_REPO", self.repo.as_smol_str());

        if cfg!(feature = "borg1-compat") {
            command.arg(format!("::{}", self.name.as_str()));
        } else {
            command.arg(self.name.as_str());
        }

        if let Some(root) = roots.first() {
            command.current_dir(root.resolve());
        }

        command.args(roots);

        Ok(())
    }
}

impl TryFrom<&toml::Value> for Archive {
    type Error = &'static str;

    #[inline]
    fn try_from(value: &toml::Value) -> Result<Self, Self::Error> {
        use toml::Value;
        let tab = value.as_table().ok_or("Invalid archive")?;

        let repo = match tab.get("repo") {
            None => return Err("Missing repository"),
            Some(v) => Repo::try_from(v)?,
        };

        let name = match tab.get("name") {
            None => return Err("Missing archive name"),
            Some(Value::String(name)) => name,
            _ => return Err("Invalid archive name"),
        };

        let options = tab.try_into()?;

        Ok(Self {
            repo,
            name: name.into(),
            options,
        })
    }
}
