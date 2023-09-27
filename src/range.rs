use std::ops::Range;

pub const INVALID_RANGE: &str =
    "invalid range, should be 'start:end' where start and end > 0";

/// Represents either a range or a single index, parsed from a string.
///
/// A valid string is either a single integer (e.g., "5") or two integers
/// separated by a colon (e.g., "5:10"). A single integer is considered an
/// index, while two integers are considered a range's start and end.
///
/// # Examples
///
/// ```
/// # use your_crate::Parser;
/// let parser = Parser::new("5:10").unwrap();
/// assert!(parser.is_range);
/// assert_eq!(parser.range.start, 5);
/// assert_eq!(parser.range.end, 10);
/// ```
pub struct Parser {
    pub range: Range<u32>,
    pub index: u32,
    pub is_range: bool,
}

impl Parser {
    pub fn new(potential_range: &str) -> eyre::Result<Self> {
        let parts = potential_range
            .split(':')
            .filter_map(|s| s.parse().ok())
            .collect::<Vec<u32>>();

        if parts.is_empty() || parts.len() > 2 {
            return Err(eyre::eyre!(INVALID_RANGE));
        }

        let start = parts[0];

        let (range, is_range) = if parts.len() == 2 {
            let end = parts[1];

            if start >= end {
                return Err(eyre::eyre!(
                    "end cannot be less than or equal to start"
                ));
            }

            (start..end, true)
        } else {
            (start..u32::MAX, false)
        };

        let index = range.start;

        Ok(Self {
            range,
            index,
            is_range,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_range() {
        let parser = Parser::new("5:10").unwrap();
        assert_eq!(parser.range.start, 5);
        assert_eq!(parser.range.end, 10);
        assert!(parser.is_range);
    }

    #[test]
    fn test_single_index() {
        let parser = Parser::new("5").unwrap();
        assert_eq!(parser.index, 5);
        assert!(!parser.is_range);
    }

    #[test]
    fn test_invalid_range() {
        assert!(Parser::new(":").is_err());
        assert!(Parser::new("").is_err());
        assert!(Parser::new("10:5").is_err());
        assert!(Parser::new("10:10").is_err());
        assert!(Parser::new("1:10:12").is_err());
        assert!(Parser::new("a:b").is_err());
    }
}
