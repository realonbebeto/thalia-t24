use crate::base::error::ValidationError;
use error_stack::Report;
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct Username(String);

impl Username {
    pub fn parse(s: String) -> Result<Username, Report<ValidationError>> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_chars = ['/', '\\', '(', ')', '"', '<', '>', '{', '}'];

        let contains_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
            Err(Report::new(ValidationError::InvalidUsername)
                .attach(format!("Failed to parse {}", s)))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Username;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_username_is_valid() {
        let name = "ё".repeat(256);
        assert_ok!(Username::parse(name));
    }

    #[test]
    fn a_username_longer_than_32_graphemes_is_rejected() {
        let name = "a".repeat(257);
        let _ = assert_err!(Username::parse(name));
    }

    #[test]
    fn whitespace_only_name_are_rejected() {
        let name = " ".to_string();
        let _ = assert_err!(Username::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        let _ = assert_err!(Username::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '\\', '(', ')', '"', '<', '>', '{', '}', ' '] {
            let name = name.to_string();
            let _ = assert_err!(Username::parse(name));
        }
    }
    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(Username::parse(name));
    }
}
