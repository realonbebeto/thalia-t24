use crate::base::error::ValidationError;
use validator::ValidateEmail;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(s: String) -> Result<Email, anyhow::Error> {
        if s.validate_email() {
            Ok(Self(s))
        } else {
            Err(anyhow::anyhow!(ValidationError::InvalidValue {
                field: "email".into(),
                reason: "Invalid email".into(),
            }))
        }
        // TODO: add validation
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::Email;
    use claims::assert_err;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use rand::SeedableRng;

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        let _ = assert_err!(Email::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        let _ = assert_err!(Email::parse(email));
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@adomain.com".to_string();
        let _ = assert_err!(Email::parse(email));
    }
    #[derive(Debug, Clone)]

    struct ValidateEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidateEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let seed: u64 = quickcheck::Arbitrary::arbitrary(g);
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

            let email: String = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidateEmailFixture) -> bool {
        Email::parse(valid_email.0).is_ok()
    }
}
