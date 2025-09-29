use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use utoipa::ToSchema;

use crate::{authentication::encode_password, base::error::ValidationError};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct PHash(String);

impl AsRef<str> for PHash {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct Password {
    pstr: String,
    phash: PHash,
}

impl Password {
    pub fn parse(s: String) -> Result<Password, Report<ValidationError>> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_short = s.graphemes(true).count() < 8;

        if is_empty_or_whitespace || is_too_short {
            Err(Report::new(ValidationError::InvalidPassword)
                .attach(format!("{} is not a valid profile password", s)))
        } else {
            // Hashing the password
            let phash =
                encode_password(s.clone()).change_context(ValidationError::InvalidPassword)?;
            Ok(Self {
                pstr: s,
                phash: PHash(phash),
            })
        }
    }

    pub fn phash_as_ref(&self) -> &str {
        self.phash.as_ref()
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.pstr
    }
}

#[cfg(test)]
mod tests {

    use crate::base::password::Password;
    use claims::{assert_err, assert_ok};

    #[test]
    fn valid_password() {
        let s = "1".repeat(8);
        assert_ok!(Password::parse(s));
    }

    #[test]
    fn short_password_is_rejected() {
        let s = "2".repeat(4);
        let _ = assert_err!(Password::parse(s));
    }

    #[test]
    fn empty_string_is_rejected() {
        let s = "".to_string();
        let _ = assert_err!(Password::parse(s));
    }

    #[test]
    fn whitespace_are_rejected() {
        let s = " ".to_string();
        let _ = assert_err!(Password::parse(s));
    }
}
