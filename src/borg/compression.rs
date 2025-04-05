use std::str::FromStr;

use smol_str::SmolStr;

pub trait AsCompression {
    fn as_compression(&self) -> Compression;

    #[inline]
    fn as_compression_str(&self) -> SmolStr {
        self.as_compression().as_compression_str()
    }

    #[inline]
    fn level(&self) -> Option<u8> {
        self.as_compression().level()
    }

    #[inline]
    fn auto(&self) -> bool {
        self.as_compression().auto()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lz4;

impl AsCompression for Lz4 {
    #[inline]
    fn as_compression(&self) -> Compression {
        Compression::Lz4
    }

    #[inline]
    fn as_compression_str(&self) -> SmolStr {
        SmolStr::new_inline("lz4")
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

impl AsCompression for Zstd {
    #[inline]
    fn as_compression(&self) -> Compression {
        Compression::Zstd(*self)
    }

    #[inline]
    fn as_compression_str(&self) -> SmolStr {
        match *self {
            Self::Level1 => SmolStr::new_inline("zstd,1"),
            Self::Level2 => SmolStr::new_inline("zstd,2"),
            Self::Level3 => SmolStr::new_inline("zstd,3"),
            Self::Level4 => SmolStr::new_inline("zstd,4"),
            Self::Level5 => SmolStr::new_inline("zstd,5"),
            Self::Level6 => SmolStr::new_inline("zstd,6"),
            Self::Level7 => SmolStr::new_inline("zstd,7"),
            Self::Level8 => SmolStr::new_inline("zstd,8"),
            Self::Level9 => SmolStr::new_inline("zstd,9"),
            Self::Level10 => SmolStr::new_inline("zstd,10"),
            Self::Level11 => SmolStr::new_inline("zstd,11"),
            Self::Level12 => SmolStr::new_inline("zstd,12"),
            Self::Level13 => SmolStr::new_inline("zstd,13"),
            Self::Level14 => SmolStr::new_inline("zstd,14"),
            Self::Level15 => SmolStr::new_inline("zstd,15"),
            Self::Level16 => SmolStr::new_inline("zstd,16"),
            Self::Level17 => SmolStr::new_inline("zstd,17"),
            Self::Level18 => SmolStr::new_inline("zstd,18"),
            Self::Level19 => SmolStr::new_inline("zstd,19"),
            Self::Level20 => SmolStr::new_inline("zstd,20"),
            Self::Level21 => SmolStr::new_inline("zstd,21"),
            Self::Level22 => SmolStr::new_inline("zstd,22"),
        }
    }

    #[inline]
    fn level(&self) -> Option<u8> {
        Some(*self as u8)
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

impl AsCompression for Zlib {
    #[inline]
    fn as_compression(&self) -> Compression {
        Compression::Zlib(*self)
    }

    #[inline]
    fn as_compression_str(&self) -> SmolStr {
        match *self {
            Self::Level0 => SmolStr::new_inline("zlib,0"),
            Self::Level1 => SmolStr::new_inline("zlib,1"),
            Self::Level2 => SmolStr::new_inline("zlib,2"),
            Self::Level3 => SmolStr::new_inline("zlib,3"),
            Self::Level4 => SmolStr::new_inline("zlib,4"),
            Self::Level5 => SmolStr::new_inline("zlib,5"),
            Self::Level6 => SmolStr::new_inline("zlib,6"),
            Self::Level7 => SmolStr::new_inline("zlib,7"),
            Self::Level8 => SmolStr::new_inline("zlib,8"),
            Self::Level9 => SmolStr::new_inline("zlib,9"),
        }
    }

    #[inline]
    fn level(&self) -> Option<u8> {
        Some(*self as u8)
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

impl AsCompression for Lzma {
    #[inline]
    fn as_compression(&self) -> Compression {
        Compression::Lzma(*self)
    }

    #[inline]
    fn as_compression_str(&self) -> SmolStr {
        match *self {
            Self::Level0 => SmolStr::new_inline("lzma,0"),
            Self::Level1 => SmolStr::new_inline("lzma,1"),
            Self::Level2 => SmolStr::new_inline("lzma,2"),
            Self::Level3 => SmolStr::new_inline("lzma,3"),
            Self::Level4 => SmolStr::new_inline("lzma,4"),
            Self::Level5 => SmolStr::new_inline("lzma,5"),
            Self::Level6 => SmolStr::new_inline("lzma,6"),
            Self::Level7 => SmolStr::new_inline("lzma,7"),
            Self::Level8 => SmolStr::new_inline("lzma,8"),
            Self::Level9 => SmolStr::new_inline("lzma,9"),
        }
    }

    #[inline]
    fn level(&self) -> Option<u8> {
        Some(*self as u8)
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
}

impl AsCompression for Compression {
    #[inline]
    fn as_compression(&self) -> Compression {
        *self
    }

    #[inline]
    fn as_compression_str(&self) -> SmolStr {
        let prefix = "auto,".chars();
        match *self {
            Self::None => SmolStr::new_inline("none"),
            Self::Lz4 => SmolStr::new_inline("lz4"),
            Self::Lz4Auto => SmolStr::new_inline("auto,lz4"),
            Self::Zstd(l) => l.as_compression_str(),
            Self::ZstdAuto(l) => prefix.chain(l.as_compression_str().chars()).collect(),
            Self::Zlib(l) => l.as_compression_str(),
            Self::ZlibAuto(l) => prefix.chain(l.as_compression_str().chars()).collect(),
            Self::Lzma(l) => l.as_compression_str(),
            Self::LzmaAuto(l) => prefix.chain(l.as_compression_str().chars()).collect(),
        }
    }

    #[inline]
    fn auto(&self) -> bool {
        matches!(
            self,
            Self::Lz4Auto | Self::LzmaAuto(..) | Self::ZstdAuto(..) | Self::ZlibAuto(..)
        )
    }

    #[inline]
    fn level(&self) -> Option<u8> {
        match *self {
            Self::None | Self::Lz4 | Self::Lz4Auto => None,
            Self::Lzma(l) | Self::LzmaAuto(l) => l.level(),
            Self::Zlib(l) | Self::ZlibAuto(l) => l.level(),
            Self::Zstd(l) | Self::ZstdAuto(l) => l.level(),
        }
    }
}

impl core::fmt::Display for Compression {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_compression_str().as_str())
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
        assert_eq!(compression.auto(), false);
        assert_eq!(compression.to_string(), "zstd,3");
    }
}
