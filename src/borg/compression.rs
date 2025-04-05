use std::str::FromStr;

use smol_str::SmolStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lz4;

impl From<Lz4> for Compression {
    #[inline]
    fn from(_: Lz4) -> Self {
        Self::Lz4
    }
}

impl AsRef<str> for Lz4 {
    #[inline]
    fn as_ref(&self) -> &str {
        "lz4"
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Zstd {
    Level1 = 1,
    Level2,
    #[default]
    Level3,
    Level4,
    Level5,
    Level6,
    Level7,
    Level8,
    Level9,
    Level10,
    Level11,
    Level12,
    Level13,
    Level14,
    Level15,
    Level16,
    Level17,
    Level18,
    Level19,
    Level20,
    Level21,
    Level22,
}

impl Zstd {
    #[inline]
    pub const fn level(&self) -> u8 {
        *self as u8
    }
}

impl From<Zstd> for Compression {
    #[inline]
    fn from(zstd: Zstd) -> Self {
        Self::Zstd(zstd)
    }
}

impl AsRef<str> for Zstd {
    #[inline]
    fn as_ref(&self) -> &str {
        match *self {
            Zstd::Level1 => "zstd,1",
            Zstd::Level2 => "zstd,2",
            Zstd::Level3 => "zstd,3",
            Zstd::Level4 => "zstd,4",
            Zstd::Level5 => "zstd,5",
            Zstd::Level6 => "zstd,6",
            Zstd::Level7 => "zstd,7",
            Zstd::Level8 => "zstd,8",
            Zstd::Level9 => "zstd,9",
            Zstd::Level10 => "zstd,10",
            Zstd::Level11 => "zstd,11",
            Zstd::Level12 => "zstd,12",
            Zstd::Level13 => "zstd,13",
            Zstd::Level14 => "zstd,14",
            Zstd::Level15 => "zstd,15",
            Zstd::Level16 => "zstd,16",
            Zstd::Level17 => "zstd,17",
            Zstd::Level18 => "zstd,18",
            Zstd::Level19 => "zstd,19",
            Zstd::Level20 => "zstd,20",
            Zstd::Level21 => "zstd,21",
            Zstd::Level22 => "zstd,22",
        }
    }
}

impl FromStr for Zstd {
    type Err = &'static str;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "zstd" => Self::default(),
            "zstd,1" => Self::Level1,
            "zstd,2" => Self::Level2,
            "zstd,3" => Self::Level3,
            "zstd,4" => Self::Level4,
            "zstd,5" => Self::Level5,
            "zstd,6" => Self::Level6,
            "zstd,7" => Self::Level7,
            "zstd,8" => Self::Level8,
            "zstd,9" => Self::Level9,
            "zstd,10" => Self::Level10,
            "zstd,11" => Self::Level11,
            "zstd,12" => Self::Level12,
            "zstd,13" => Self::Level13,
            "zstd,14" => Self::Level14,
            "zstd,15" => Self::Level15,
            "zstd,16" => Self::Level16,
            "zstd,17" => Self::Level17,
            "zstd,18" => Self::Level18,
            "zstd,19" => Self::Level19,
            "zstd,20" => Self::Level20,
            "zstd,21" => Self::Level21,
            "zstd,22" => Self::Level22,
            _ => return Err("invalid zstd compression string"),
        })
    }
}

impl TryFrom<u8> for Zstd {
    type Error = &'static str;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Level1,
            2 => Self::Level2,
            3 => Self::Level3,
            4 => Self::Level4,
            5 => Self::Level5,
            6 => Self::Level6,
            7 => Self::Level7,
            8 => Self::Level8,
            9 => Self::Level9,
            10 => Self::Level10,
            11 => Self::Level11,
            12 => Self::Level12,
            13 => Self::Level13,
            14 => Self::Level14,
            15 => Self::Level15,
            16 => Self::Level16,
            17 => Self::Level17,
            18 => Self::Level18,
            19 => Self::Level19,
            20 => Self::Level20,
            21 => Self::Level21,
            22 => Self::Level22,
            _ => return Err("invalid zstd compression level"),
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Zlib {
    Level0 = 0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
    #[default]
    Level6,
    Level7,
    Level8,
    Level9,
}

impl Zlib {
    #[inline]
    pub const fn level(&self) -> u8 {
        *self as u8
    }
}

impl From<Zlib> for Compression {
    #[inline]
    fn from(zlib: Zlib) -> Self {
        Self::Zlib(zlib)
    }
}

impl AsRef<str> for Zlib {
    #[inline]
    fn as_ref(&self) -> &str {
        match *self {
            Zlib::Level0 => "zlib,0",
            Zlib::Level1 => "zlib,1",
            Zlib::Level2 => "zlib,2",
            Zlib::Level3 => "zlib,3",
            Zlib::Level4 => "zlib,4",
            Zlib::Level5 => "zlib,5",
            Zlib::Level6 => "zlib,6",
            Zlib::Level7 => "zlib,7",
            Zlib::Level8 => "zlib,8",
            Zlib::Level9 => "zlib,9",
        }
    }
}

impl FromStr for Zlib {
    type Err = &'static str;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "zlib" => Self::default(),
            "zlib,0" => Self::Level0,
            "zlib,1" => Self::Level1,
            "zlib,2" => Self::Level2,
            "zlib,3" => Self::Level3,
            "zlib,4" => Self::Level4,
            "zlib,5" => Self::Level5,
            "zlib,6" => Self::Level6,
            "zlib,7" => Self::Level7,
            "zlib,8" => Self::Level8,
            "zlib,9" => Self::Level9,
            _ => return Err("invalid zlib compression string"),
        })
    }
}

impl TryFrom<u8> for Zlib {
    type Error = &'static str;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Level0,
            1 => Self::Level1,
            2 => Self::Level2,
            3 => Self::Level3,
            4 => Self::Level4,
            5 => Self::Level5,
            6 => Self::Level6,
            7 => Self::Level7,
            8 => Self::Level8,
            9 => Self::Level9,
            _ => return Err("invalid zlib compression level"),
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Lzma {
    Level0 = 0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
    #[default]
    Level6,
    Level7,
    Level8,
    Level9,
}

impl Lzma {
    #[inline]
    pub const fn level(&self) -> u8 {
        *self as u8
    }
}

impl From<Lzma> for Compression {
    #[inline]
    fn from(lzma: Lzma) -> Self {
        Self::Lzma(lzma)
    }
}

impl AsRef<str> for Lzma {
    #[inline]
    fn as_ref(&self) -> &str {
        match *self {
            Lzma::Level0 => "lzma,0",
            Lzma::Level1 => "lzma,1",
            Lzma::Level2 => "lzma,2",
            Lzma::Level3 => "lzma,3",
            Lzma::Level4 => "lzma,4",
            Lzma::Level5 => "lzma,5",
            Lzma::Level6 => "lzma,6",
            Lzma::Level7 => "lzma,7",
            Lzma::Level8 => "lzma,8",
            Lzma::Level9 => "lzma,9",
        }
    }
}

impl FromStr for Lzma {
    type Err = &'static str;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "lzma" => Self::default(),
            "lzma,0" => Self::Level0,
            "lzma,1" => Self::Level1,
            "lzma,2" => Self::Level2,
            "lzma,3" => Self::Level3,
            "lzma,4" => Self::Level4,
            "lzma,5" => Self::Level5,
            "lzma,6" => Self::Level6,
            "lzma,7" => Self::Level7,
            "lzma,8" => Self::Level8,
            "lzma,9" => Self::Level9,
            _ => return Err("invalid lzma compression string"),
        })
    }
}

impl TryFrom<u8> for Lzma {
    type Error = &'static str;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Level0,
            1 => Self::Level1,
            2 => Self::Level2,
            3 => Self::Level3,
            4 => Self::Level4,
            5 => Self::Level5,
            6 => Self::Level6,
            7 => Self::Level7,
            8 => Self::Level8,
            9 => Self::Level9,
            _ => return Err("invalid lzma compression level"),
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Compression {
    None,
    #[default]
    Lz4,
    Lz4Auto,
    Zstd(Zstd),
    ZstdAuto(Zstd),
    Zlib(Zlib),
    ZlibAuto(Zlib),
    Lzma(Lzma),
    LzmaAuto(Lzma),
}

impl Compression {
    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub const fn is_auto(&self) -> bool {
        matches!(
            self,
            Self::Lz4Auto | Self::LzmaAuto(..) | Self::ZstdAuto(..) | Self::ZlibAuto(..)
        )
    }

    #[inline]
    pub const fn level(&self) -> Option<u8> {
        match *self {
            Self::None | Self::Lz4 | Self::Lz4Auto => None,
            Self::Lzma(l) | Self::LzmaAuto(l) => Some(l.level()),
            Self::Zlib(l) | Self::ZlibAuto(l) => Some(l.level()),
            Self::Zstd(l) | Self::ZstdAuto(l) => Some(l.level()),
        }
    }

    #[inline]
    pub fn as_smol_str(&self) -> SmolStr {
        let prefix = ['a', 'u', 't', 'o', ','].into_iter();
        match *self {
            Self::None => SmolStr::new_static("none"),
            Self::Lz4 => SmolStr::new_static("lz4"),
            Self::Lz4Auto => SmolStr::new_static("auto,lz4"),
            Self::Zstd(l) => SmolStr::new_inline(l.as_ref()),
            Self::ZstdAuto(l) => prefix.chain(l.as_ref().chars()).collect(),
            Self::Zlib(l) => SmolStr::new_inline(l.as_ref()),
            Self::ZlibAuto(l) => prefix.chain(l.as_ref().chars()).collect(),
            Self::Lzma(l) => SmolStr::new_inline(l.as_ref()),
            Self::LzmaAuto(l) => prefix.chain(l.as_ref().chars()).collect(),
        }
    }
}

impl core::fmt::Display for Compression {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_smol_str().fmt(f)
    }
}

impl std::str::FromStr for Compression {
    type Err = &'static str;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "lz4" => Ok(Self::Lz4),
            "auto,lz4" => Ok(Self::Lz4Auto),
            "lzma" => Ok(Self::Lzma(Lzma::default())),
            "auto,lzma" => Ok(Self::LzmaAuto(Lzma::default())),
            "zlib" => Ok(Self::Zlib(Zlib::default())),
            "auto,zlib" => Ok(Self::ZlibAuto(Zlib::default())),
            "zstd" => Ok(Self::Zstd(Zstd::default())),
            "auto,zstd" => Ok(Self::ZstdAuto(Zstd::default())),
            c if c.starts_with("lzma,") => Ok(Self::Lzma(c.parse()?)),
            c if c.starts_with("auto,lzma,") => Ok(c[5..].parse()?),
            c if c.starts_with("zlib,") => Ok(Self::Zlib(c.parse()?)),
            c if c.starts_with("auto,zlib,") => Ok(c[5..].parse()?),
            c if c.starts_with("zstd,") => Ok(Self::Zstd(c.parse()?)),
            c if c.starts_with("auto,zstd,") => Ok(c[5..].parse()?),
            _ => Err("Invalid compression format"),
        }
    }
}

impl TryFrom<&toml::Value> for Compression {
    type Error = &'static str;

    #[inline]
    fn try_from(value: &toml::Value) -> Result<Self, Self::Error> {
        if let Some(s) = value.as_str() {
            return s.parse();
        }

        let Some(table) = value.as_table() else {
            return Err("Invalid compression format");
        };

        let Some(algorithm) = table.get("algorithm").and_then(toml::Value::as_str) else {
            return Err("Missing algorithm key");
        };

        let level = table
            .get("level")
            .and_then(toml::Value::as_integer)
            .and_then(|l| u8::try_from(l).ok());

        let auto = table
            .get("auto")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);

        let comp = match (algorithm, auto, level) {
            ("none", ..) => Self::None,
            ("lz4", false, ..) => Self::Lz4,
            ("lz4", true, ..) => Self::Lz4Auto,
            ("lzma", false, Some(l)) => Self::Lzma(l.try_into()?),
            ("lzma", true, Some(l)) => Self::LzmaAuto(l.try_into()?),
            ("zlib", false, Some(l)) => Self::Zlib(l.try_into()?),
            ("zlib", true, Some(l)) => Self::ZlibAuto(l.try_into()?),
            ("zstd", false, Some(l)) => Self::Zstd(l.try_into()?),
            ("zstd", true, Some(l)) => Self::ZstdAuto(l.try_into()?),
            ("zstd", false, ..) => Self::Zstd(Zstd::default()),
            ("zstd", true, ..) => Self::ZstdAuto(Zstd::default()),
            ("lzma", false, ..) => Self::Lzma(Lzma::default()),
            ("lzma", true, ..) => Self::LzmaAuto(Lzma::default()),
            ("zlib", false, ..) => Self::Zlib(Zlib::default()),
            ("zlib", true, ..) => Self::ZlibAuto(Zlib::default()),
            _ => return Err("Invalid compression algorithm"),
        };

        Ok(comp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression() {
        let compression = Compression::Zstd(Zstd::Level3);
        assert_eq!(compression.level(), Some(3));
        assert_eq!(compression.is_auto(), false);
        assert_eq!(compression.to_string(), "zstd,3");
    }
}
