use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};

static HOME: LazyLock<Option<PathBuf>> = LazyLock::new(dirs::home_dir);

pub trait PathResolveExt: AsRef<Path> {
    #[inline]
    fn resolve(&self) -> Cow<Path> {
        let path = self.as_ref();
        if path == Path::new("~") {
            return Cow::Borrowed(HOME.as_ref().unwrap());
        }

        if let Ok(path) = path.strip_prefix("~/") {
            return Cow::Owned(HOME.as_ref().unwrap().join(path));
        }

        Cow::Borrowed(path)
    }
}

impl<T: AsRef<Path>> PathResolveExt for T {}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseByteSizeError {
    #[error("Invalid suffix '{}'", 0 as char)]
    InvalidSuffix(u8),
    #[error(transparent)]
    InvalidInt(#[from] std::num::ParseFloatError),
    #[error("Invalid UTF8")]
    Utf8Error,
    #[error("Negative byte size")]
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteSize(pub u64);

impl FromStr for ByteSize {
    type Err = ParseByteSizeError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ascii = s.as_bytes();
        let (num, fac) = match ascii {
            [num @ .., b'K' | b'k'] => (num, 1_000f64),
            [num @ .., b'M'] => (num, 1_000_000f64),
            [num @ .., b'G'] => (num, 1_000_000_000f64),
            [num @ .., b'T'] => (num, 1_000_000_000_000f64),
            [num @ .., b'P'] => (num, 1_000_000_000_000_000f64),
            [num @ .., b'E'] => (num, 1_000_000_000_000_000_000f64),
            [.., s] if !s.is_ascii_digit() => return Err(ParseByteSizeError::InvalidSuffix(*s)),
            num => (num, 0f64),
        };

        let Ok(num) = std::str::from_utf8(num) else {
            return Err(ParseByteSizeError::Utf8Error);
        };

        let num = num.parse::<f64>()?;
        let num = num * fac;
        if num.is_sign_negative() {
            return Err(ParseByteSizeError::Negative);
        }
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let num = num as u64;
        Ok(Self(num))
    }
}

impl ByteSize {
    #[allow(clippy::cast_precision_loss)]
    #[inline]
    pub fn fmt_iec(self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Debug;
        const KIBI: u64 = 1024;
        const MEBI: u64 = KIBI * 1024;
        const GIBI: u64 = MEBI * 1024;
        const TEBI: u64 = GIBI * 1024;
        const PEBI: u64 = TEBI * 1024;
        const EXBI: u64 = PEBI * 1024;

        match self.0 {
            b @ ..KIBI => b.fmt(f),
            EXBI => f.write_str("1Ei"),
            b @ EXBI.. => {
                (b as f64 / 1_152_921_504_606_846_976f64).fmt(f)?;
                f.write_str("Ei")
            }
            PEBI => f.write_str("1Pi"),
            b @ PEBI.. => {
                (b as f64 / 1_125_899_906_842_624f64).fmt(f)?;
                f.write_str("Pi")
            }
            TEBI => f.write_str("1Ti"),
            b @ TEBI.. => {
                (b as f64 / 1_099_511_627_776f64).fmt(f)?;
                f.write_str("Ti")
            }
            GIBI => f.write_str("1Gi"),
            b @ GIBI.. => {
                (b as f64 / 1_073_741_824f64).fmt(f)?;
                f.write_str("Gi")
            }
            MEBI => f.write_str("1Mi"),
            b @ MEBI.. => {
                (b as f64 / 1_048_576f64).fmt(f)?;
                f.write_str("Mi")
            }
            KIBI => f.write_str("1Ki"),
            b @ KIBI.. => {
                (b as f64 / 1024f64).fmt(f)?;
                f.write_str("Ki")
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    #[inline]
    pub fn fmt_si(self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Debug;
        const KILO: u64 = 1000;
        const MEGA: u64 = KILO * 1000;
        const GIGA: u64 = MEGA * 1000;
        const TERA: u64 = GIGA * 1000;
        const PETA: u64 = TERA * 1000;
        const EXA: u64 = PETA * 1000;

        match self.0 {
            b @ ..KILO => b.fmt(f),
            EXA => f.write_str("1E"),
            b @ EXA.. => {
                (b as f64 / 1_000_000_000_000_000_000f64).fmt(f)?;
                f.write_str("E")
            }
            PETA => f.write_str("1P"),
            b @ PETA.. => {
                (b as f64 / 1_000_000_000_000_000f64).fmt(f)?;
                f.write_str("P")
            }
            TERA => f.write_str("1T"),
            b @ TERA.. => {
                (b as f64 / 1_000_000_000_000f64).fmt(f)?;
                f.write_str("TB")
            }
            GIGA => f.write_str("1G"),
            b @ GIGA.. => {
                (b as f64 / 1_000_000_000f64).fmt(f)?;
                f.write_str("G")
            }
            MEGA => f.write_str("1M"),
            b @ MEGA.. => {
                (b as f64 / 1_000_000f64).fmt(f)?;
                f.write_str("M")
            }
            KILO => f.write_str("1k"),
            b @ KILO.. => {
                (b as f64 / 1000f64).fmt(f)?;
                f.write_str("k")
            }
        }
    }
}

impl std::fmt::Display for ByteSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            self.fmt_si(f)
        } else {
            self.fmt_iec(f)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path() {
        let should_resolve = PathBuf::from("~/test");
        assert_ne!(should_resolve, should_resolve.resolve());

        let should_not_resolve = PathBuf::from("/test");
        assert_eq!(should_not_resolve, should_not_resolve.resolve());

        let should_not_resolve = PathBuf::from("~test");
        assert_eq!(should_not_resolve, should_not_resolve.resolve());

        let home_only = PathBuf::from("~");
        assert_ne!(home_only, home_only.resolve());
    }

    #[test]
    fn test_byte_size() {
        const ZERO: ByteSize = ByteSize(0);
        const ONE: ByteSize = ByteSize(1);
        const TEN: ByteSize = ByteSize(10);
        const HUNDRED: ByteSize = ByteSize(100);
        const THOUSAND: ByteSize = ByteSize(1000);
        const KIBI: ByteSize = ByteSize(1024);
        const TEN25: ByteSize = ByteSize(1025);
        const MEGA: ByteSize = ByteSize(1024 * 1024);

        assert_eq!(format!("{ZERO:}"), "0");
        assert_eq!(format!("{ZERO:#}"), "0");
        assert_eq!(format!("{ONE:}"), "1");
        assert_eq!(format!("{ONE:#}"), "1");
        assert_eq!(format!("{TEN:}"), "10");
        assert_eq!(format!("{TEN:#}"), "10");
        assert_eq!(format!("{HUNDRED:}"), "100");
        assert_eq!(format!("{HUNDRED:#}"), "100");
        assert_eq!(format!("{THOUSAND:}"), "1000");
        assert_eq!(format!("{THOUSAND:#}"), "1k");
        assert_eq!(format!("{KIBI:}"), "1Ki");
        assert_eq!(format!("{KIBI:#}"), "1.024k");
        assert_eq!(format!("{TEN25:.3}"), "1.001Ki");
        assert_eq!(format!("{TEN25:#}"), "1.025k");
        assert_eq!(format!("{MEGA:}"), "1Mi");
        assert_eq!(format!("{MEGA:#.3}"), "1.049M");
    }
}
