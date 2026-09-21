use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagParseError {
    Empty,
}

impl fmt::Display for TagParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("tag must not be blank")
    }
}

impl std::error::Error for TagParseError {}

macro_rules! tag_type {
    ($name:ident) => {
        /// A nonempty, trimmed identifier. Case and leading zeros are preserved.
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
                Ok(Self(value.to_owned()))
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

tag_type!(Tag);
tag_type!(TipTag);
tag_type!(UhfTag);
tag_type!(UhfTagVisual);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_preserves_case_and_leading_zeros() {
        assert_eq!(" 000aB ".parse::<Tag>().unwrap().as_str(), "000aB");
        assert_eq!(" TIP-01 ".parse::<TipTag>().unwrap().as_str(), "TIP-01");
        assert_eq!(" 00aB ".parse::<UhfTag>().unwrap().as_str(), "00aB");
        assert_eq!(" 0001 ".parse::<UhfTagVisual>().unwrap().as_str(), "0001");
    }

    #[test]
    fn blank_identifiers_are_rejected() {
        assert_eq!(" \t".parse::<Tag>(), Err(TagParseError::Empty));
        assert_eq!("".parse::<TipTag>(), Err(TagParseError::Empty));
        assert_eq!(" ".parse::<UhfTag>(), Err(TagParseError::Empty));
        assert_eq!("\n".parse::<UhfTagVisual>(), Err(TagParseError::Empty));
    }
}
