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
        self.options.apply(command);

        command.env("BORG_REPO", self.repo.as_smol_str());

        if cfg!(feature = "borg1-compat") {
            command.arg(format!("::{}", self.name.as_str()));
        } else {
            command.arg(self.name.as_str());
        }

        let mut roots = self.roots().map(PathResolveExt::resolve);

        let cwd;

        if let Some(root) = roots.next() {
            command.arg(root.as_ref());
            for r in roots {
                command.arg(r.as_ref());
            }
            // First root sets the current directory
            cwd = root;
        } else {
            let home = "~".resolve();
            command.arg(home.as_ref());
            cwd = home;
        }

        if cwd.join(".borgignore").is_file() {
            command.arg("--exclude-from");
            command.arg(".borgignore");
        }

        command.current_dir(cwd);

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
