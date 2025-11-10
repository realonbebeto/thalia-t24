use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core},
};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use utoipa::ToSchema;

use crate::base::error::{AuthError, ValidationError};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct Password {
    raw: String,
}

impl Password {
    pub fn parse(s: String) -> Result<Password, anyhow::Error> {
        if s.trim().is_empty() {
            Err(anyhow::anyhow!(ValidationError::MissingField(
                "Password is null/empty".into(),
            )))?
        }
        let common_passwords = [
            "password",
            "password123",
            "12345678",
            "qwerty123",
            "admin123",
            "letmein1",
            "welcome1",
            "Password1",
        ];

        if common_passwords.iter().any(|c| c == &s.as_str()) {
            Err(anyhow::anyhow!(ValidationError::InvalidValue {
                field: "password".into(),
                reason: "Too common for the security".into(),
            }))?
        }

        if s.graphemes(true).count() < 8 {
            Err(anyhow::anyhow!(ValidationError::TooShort {
                field: "password".into(),
                min: 8,
            }))?
        }

        if s.graphemes(true)
            .all(|c| c.chars().all(|c| c.is_lowercase()))
        {
            Err(anyhow::anyhow!(ValidationError::InvalidValue {
                field: "password".into(),
                reason: "All characters should not be lowercase".into(),
            }))?
        }

        if s.graphemes(true)
            .all(|c| c.chars().all(|c| c.is_uppercase()))
        {
            Err(anyhow::anyhow!(ValidationError::InvalidValue {
                field: "password".into(),
                reason: "All characters should not be uppercase".into(),
            }))?
        }

        if s.graphemes(true)
            .any(|c| c.chars().any(|c| c.is_whitespace()))
        {
            Err(anyhow::anyhow!(ValidationError::InvalidValue {
                field: "password".into(),
                reason: "Password should not have whitespace".into(),
            }))?
        }

        if s.graphemes(true).all(|c| c.chars().all(|c| c.is_numeric())) {
            Err(anyhow::anyhow!(ValidationError::InvalidValue {
                field: "password".into(),
                reason: "All characters should not be numeric".into(),
            }))?
        }

        if s.graphemes(true)
            .all(|c| c.chars().all(|c| c.is_alphabetic()))
        {
            Err(anyhow::anyhow!(ValidationError::InvalidValue {
                field: "password".into(),
                reason: "All characters should not be only alphabetic".into(),
            }))?
        }

        if !s.graphemes(true).any(|c| c.chars().any(|c| c.is_numeric())) {
            Err(anyhow::anyhow!(ValidationError::InvalidValue {
                field: "password".into(),
                reason: "Password should include numeric values".into(),
            }))?
        }

        Ok(Self { raw: s })
    }

    pub fn encode_password(&self) -> Result<String, anyhow::Error> {
        let salt = SaltString::generate(&mut rand_core::OsRng);
        let password = Argon2::new(
            argon2::Algorithm::Argon2id,
            Version::V0x13,
            Params::new(27000, 2, 1, None)?,
        )
        .hash_password(self.raw.as_bytes(), &salt)?
        .to_string();

        Ok(password)
    }

    pub fn verify_password(expected_password: &str, password: &str) -> Result<(), anyhow::Error> {
        let expected_password = PasswordHash::new(expected_password).map_err(|e| {
            anyhow::anyhow!(AuthError::InvalidCredentials("Password or Username".into())).context(e)
        })?;

        Argon2::default()
            .verify_password(password.as_bytes(), &expected_password)
            .map_err(|e| {
                anyhow::anyhow!(AuthError::InvalidCredentials("Password or Username".into()))
                    .context(e)
            })?;

        Ok(())
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

#[cfg(test)]
mod tests {

    use crate::base::password::Password;
    use claims::{assert_err, assert_ok};

    #[test]
    fn valid_password() {
        let s = "Password#123$17173630>".repeat(8);
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
