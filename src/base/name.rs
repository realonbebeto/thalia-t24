use crate::base::error::ValidationError;
use error_stack::Report;
use unicode_segmentation::UnicodeSegmentation;
/// Name
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Name(String);

impl Name {
    pub fn parse(s: String) -> Result<Name, Report<ValidationError>> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 256;

        let is_too_short = s.graphemes(true).count() < 3;

        let forbidden_chars = ['/', '\\', '(', ')', '"', '<', '>', '{', '}'];

        let contains_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));

        if is_empty_or_whitespace || is_too_long || is_too_short || contains_forbidden_chars {
            Err(Report::new(ValidationError::InvalidName))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Name;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "ё".repeat(256);
        assert_ok!(Name::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        let _ = assert_err!(Name::parse(name));
    }

    #[test]
    fn whitespace_only_name_are_rejected() {
        let name = " ".to_string();
        let _ = assert_err!(Name::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        let _ = assert_err!(Name::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '\\', '(', ')', '"', '<', '>', '{', '}'] {
            let name = name.to_string();
            let _ = assert_err!(Name::parse(name));
        }
    }
    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(Name::parse(name));
    }
}
