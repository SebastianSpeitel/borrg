use thiserror::Error;

#[derive(Error, Debug)]
pub(super) enum InvalidByteSize {
    #[error("Invalid byte size: {0}")]
    Size(String),
    #[error("Invalid byte suffix: {0}")]
    Suffix(String),
}

pub(super) fn parse_byte_size(size: &str) -> Result<u64, InvalidByteSize> {
    let (num, suffix) = size.chars().partition::<String, _>(|c| c.is_ascii_digit());

    let num: u64 = num.parse().map_err(|_| InvalidByteSize::Size(num))?;

    let factor = match suffix.as_str() {
        "" => 1,
        "K" => 1024,
        "M" => 1024 * 1024,
        "G" => 1024 * 1024 * 1024,
        "T" => 1024 * 1024 * 1024 * 1024,
        "P" => 1024 * 1024 * 1024 * 1024 * 1024,
        _ => return Err(InvalidByteSize::Suffix(suffix)),
    };
    Ok(num * factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_byte_size() {
        assert_eq!(parse_byte_size("1").unwrap(), 1);
        assert_eq!(parse_byte_size("1K").unwrap(), 1024);
        assert_eq!(parse_byte_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_byte_size("1G").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_byte_size("1T").unwrap(), 1024 * 1024 * 1024 * 1024);
        assert_eq!(
            parse_byte_size("1P").unwrap(),
            1024 * 1024 * 1024 * 1024 * 1024
        );

        assert!(parse_byte_size("1X").is_err());
        assert!(parse_byte_size("X").is_err());
    }
}
