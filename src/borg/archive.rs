use smol_str::SmolStr;

use crate::util::PathResolveExt;

use super::{Compression, Repo};

#[derive(Debug, Clone)]
pub struct Archive {
    repo: Repo<'static>,
    name: SmolStr,
    options: Options,
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
            }
        }

        command.env("BORG_REPO", self.repo.as_smol_str());

        if cfg!(feature = "borg1") {
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

#[derive(Debug, Clone, Default)]
pub struct Options(Vec<Opt>);

#[derive(Debug, Clone)]
enum Opt {
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
                (
                    "exclude_caches" | "exclude_nodump" | "one_file_system",
                    Value::Boolean(false),
                ) => {
                    // Allowed but ignored
                }
                ("comment", Value::String(s)) => {
                    opts.push(Opt::Comment(s.into()));
                }
                ("compression", val) => {
                    opts.push(Opt::Compression(Compression::try_from(val)?));
                }
                _ => return Err("Invalid archive option"),
            }
        }

        Ok(Self(opts))
    }
}

impl TryFrom<&crate::cli::BackupConfig> for Archive {
    type Error = crate::cli::ConfigError;
    fn try_from(config: &crate::cli::BackupConfig) -> Result<Self, Self::Error> {
        let name = chrono::Local::now().format("%Y-%m-%d").to_string();

        let repo = config
            .repo
            .as_ref()
            .ok_or(crate::cli::ConfigError::Other("missing repo"))?;

        let mut options = Vec::with_capacity(config.paths.len() + 3);

        for path in &config.paths {
            options.push(Opt::Root(path.to_string_lossy().into()));
        }

        if let Some(c) = config.compression {
            options.push(Opt::Compression(c));
        }

        if let Some(ref pattern_file) = config.pattern_file {
            options.push(Opt::PatternFrom(pattern_file.to_string_lossy().into()));
        }

        if let Some(ref exclude_file) = config.exclude_file {
            options.push(Opt::ExcludeFrom(exclude_file.to_string_lossy().into()));
        }

        Ok(Self {
            repo: repo.to_owned(),
            name: name.into(),
            options: Options(options),
        })
    }
}
