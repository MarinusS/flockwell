use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagParseError {
    Empty,
    InvalidLength { expected: usize, actual: usize },
    NonDigit,
}

impl fmt::Display for TagParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("tag must not be blank"),
            Self::InvalidLength { expected, actual } => write!(
                f,
                "tag must contain {expected} digits, got {actual} characters"
            ),
            Self::NonDigit => f.write_str("tag must contain only ASCII digits (0-9)"),
        }
    }
}

impl std::error::Error for TagParseError {}

macro_rules! tag_type {
    ($(#[$meta:meta])* $name:ident, $normalize:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl FromStr for $name {
            type Err = TagParseError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let value = value.trim();
                if value.is_empty() {
                    return Err(TagParseError::Empty);
                }
                Ok(Self($normalize(value)?))
            }
        }

        impl TryFrom<&str> for $name {
            type Error = TagParseError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                value.parse()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

fn numeric_tag(value: &str) -> Result<String, TagParseError> {
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(TagParseError::NonDigit);
    }
    if value.len() != Tag::LENGTH {
        return Err(TagParseError::InvalidLength {
            expected: Tag::LENGTH,
            actual: value.len(),
        });
    }
    Ok(value.to_owned())
}

fn case_insensitive_tag(value: &str) -> Result<String, TagParseError> {
    Ok(value.to_uppercase())
}

tag_type!(
    /// Exactly 15 ASCII digits, trimmed, with leading zeros preserved.
    Tag, numeric_tag
);
impl Tag {
    pub const LENGTH: usize = 15;
}
tag_type!(
    /// A nonblank, trimmed identifier stored in uppercase. Duplicates are allowed for now.
    TipTag, case_insensitive_tag
);
tag_type!(
    /// A nonblank, trimmed identifier stored in uppercase for case-insensitive uniqueness.
    UhfTag, case_insensitive_tag
);
tag_type!(
    /// A nonblank, trimmed identifier stored in uppercase. Does not require uniqueness.
    UhfTagVisual, case_insensitive_tag
);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn numeric_tag_accepts_example_and_preserves_leading_zeros() {
        assert_eq!(
            "250029228122437".parse::<Tag>().unwrap().as_str(),
            "250029228122437"
        );
        assert_eq!(
            " 000000000000042 ".parse::<Tag>().unwrap().as_str(),
            "000000000000042"
        );
    }

    #[test]
    fn numeric_tag_rejects_wrong_lengths_and_non_digits() {
        for (input, actual) in [
            ("00042", 5),
            ("25002922812243", 14),
            ("2500292281224370", 16),
        ] {
            assert_eq!(
                input.parse::<Tag>(),
                Err(TagParseError::InvalidLength {
                    expected: 15,
                    actual
                })
            );
        }
        for input in [
            "25002922812243X",
            "250029 28122437",
            "+50029228122437",
            "２５００２９２２８１２２４３７",
        ] {
            assert_eq!(input.parse::<Tag>(), Err(TagParseError::NonDigit));
        }
    }

    #[test]
    fn other_tags_normalize_case_without_format_restrictions() {
        assert_eq!(" tip-01 ".parse::<TipTag>().unwrap().as_str(), "TIP-01");
        assert_eq!(" 00aB ".parse::<UhfTag>().unwrap().as_str(), "00AB");
        assert_eq!(
            " visual label ".parse::<UhfTagVisual>().unwrap().as_str(),
            "VISUAL LABEL"
        );
        assert_eq!("tip".parse::<TipTag>(), "TIP".parse::<TipTag>());
        assert_eq!(
            "visual".parse::<UhfTagVisual>(),
            "VISUAL".parse::<UhfTagVisual>()
        );
        let tags = [
            "abc".parse::<UhfTag>().unwrap(),
            "ABC".parse::<UhfTag>().unwrap(),
        ];
        assert_eq!(tags.iter().collect::<HashSet<_>>().len(), 1);
        assert_eq!(tags.iter().collect::<BTreeSet<_>>().len(), 1);
    }

    #[test]
    fn blank_identifiers_are_rejected() {
        assert_eq!(" \t".parse::<Tag>(), Err(TagParseError::Empty));
        assert_eq!("".parse::<TipTag>(), Err(TagParseError::Empty));
        assert_eq!(" ".parse::<UhfTag>(), Err(TagParseError::Empty));
        assert_eq!("\n".parse::<UhfTagVisual>(), Err(TagParseError::Empty));
    }
}
