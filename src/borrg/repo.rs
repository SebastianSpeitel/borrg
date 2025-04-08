use crate::borg::Repo;

use super::Passphrase;
use std::{fmt::Display, str::FromStr};


#[derive(Debug, Clone, Eq)]
pub struct RepoConfig {
    pub url: Repo<'static>,
    pub(crate) passphrase: Option<Passphrase>,
}

impl FromStr for RepoConfig {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = s.parse()?;
        Ok(Self {
            url,
            passphrase: None,
        })
    }
}

impl Display for RepoConfig {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.url.fmt(f)
    }
}

impl PartialEq for RepoConfig {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}
