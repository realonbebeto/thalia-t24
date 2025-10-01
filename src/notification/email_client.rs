use crate::base::Email;
use crate::notification::schemas::{Recipient, SendEmailRequest};
use crate::notification::schemas::{WelcomeEmailTemplate, WelcomeEmailTemplateTxt};
use crate::telemetry::TraceError;
use askama::Template;
use error_stack::{Report, ResultExt};
use reqwest::{Client, Url};

#[derive(Clone, Debug)]
pub struct EmailClient {
    http_client: Client,
    email_base_uri: Url,
    sender: Email,
    private_email_key: String,
    public_email_key: String,
}

#[derive(Debug, thiserror::Error)]
pub enum EmailClientError {
    #[error("Failed to parse uri")]
    BadUri,
    #[error("Failed to build client")]
    BuildError,
    #[error("Error while trying to send welcome email")]
    WelcomeError,
    #[error("reqwest Error while sending email")]
    PostError,
}

impl EmailClient {
    pub fn new(
        email_base_uri: &str,
        sender: Email,
        private_email_key: &str,
        public_email_key: &str,
        timeout: std::time::Duration,
    ) -> Result<Self, Report<EmailClientError>> {
        let email_base_uri = Url::parse(email_base_uri)
            .trace_with(&format!(
                "Failed to parse email base uri: {}",
                email_base_uri
            ))
            .change_context(EmailClientError::BadUri)?;

        let http_client = Client::builder()
            .timeout(timeout)
            .build()
            .trace_with("Failed to build http client")
            .change_context(EmailClientError::BuildError)?;

        Ok(Self {
            http_client,
            email_base_uri,
            sender,
            private_email_key: private_email_key.to_string(),
            public_email_key: public_email_key.to_string(),
        })
    }

    async fn send_email(
        &self,
        recipient: Email,
        subject: &str,
        html_part: &str,
        text_part: &str,
    ) -> Result<(), Report<EmailClientError>> {
        let request_body = SendEmailRequest::new(
            self.sender.as_ref(),
            "Thalia Corp",
            subject,
            text_part,
            html_part,
            vec![Recipient::new(recipient.as_ref())],
        );

        self.http_client
            .post(self.email_base_uri.clone())
            .basic_auth(
                self.public_email_key.clone(),
                Some(self.private_email_key.clone()),
            )
            .json(&request_body)
            .send()
            .await
            .trace_with("reqwest error")
            .change_context(EmailClientError::PostError)?
            .error_for_status()
            .trace_with("reqwest error")
            .change_context(EmailClientError::PostError)?;

        Ok(())
    }

    pub async fn send_welcome_email(
        &self,
        app_address: &str,
        recipient: Email,
        subject: &str,
        first_name: &str,
        activate_token: &str,
        company_name: &str,
    ) -> Result<(), Report<EmailClientError>> {
        let confirmation_link = format!("{}/confirm/{}", app_address, activate_token);
        let welcome_email = WelcomeEmailTemplate::new(first_name, &confirmation_link, company_name)
            .render()
            .trace_with("Failed to render html template")
            .change_context(EmailClientError::WelcomeError)?;

        let welcome_email_txt =
            WelcomeEmailTemplateTxt::new(first_name, &confirmation_link, company_name)
                .render()
                .trace_with("Failed to rendeer txt template")
                .change_context(EmailClientError::WelcomeError)?;

        self.send_email(recipient, subject, &welcome_email, &welcome_email_txt)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::base::Email;
    use crate::notification::email_client::EmailClient;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::{
        Mock, MockServer, Request, ResponseTemplate,
        matchers::{any, basic_auth, header, method, path},
    };

    // Generate random email subject
    fn subject() -> String {
        Sentence(1..3).fake()
    }

    // Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    // Generate a random email
    fn email() -> Email {
        Email::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_uri: String) -> (EmailClient, String, String) {
        let prek = Faker.fake::<String>();
        let puek = Faker.fake::<String>();

        (
            EmailClient::new(
                &base_uri,
                email(),
                &prek,
                &puek,
                std::time::Duration::from_millis(200),
            )
            .unwrap(),
            puek,
            prek,
        )
    }

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                body.get("FromName").is_some()
                    && body.get("Recipients").is_some()
                    && body.get("Subject").is_some()
                    && body.get("Html-part").is_some()
                    && body.get("Text-part").is_some()
            } else {
                false
            }
        }
    }

    #[actix_web::test]
    async fn send_email_sends_the_expected_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let (email_client, puek, prek) = email_client(format!("{}/v3/send", mock_server.uri()));

        Mock::given(basic_auth(puek, prek))
            .and(header("Content-Type", "application/json"))
            .and(path("/v3/send"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;
    }

    #[actix_web::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let (email_client, _, _) = email_client(format!("{}/v3/send", mock_server.uri()));

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(outcome);
    }

    #[actix_web::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let (email_client, _, _) = email_client(format!("{}/v3/send", mock_server.uri()));

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        let _ = assert_err!(outcome);
    }

    #[actix_web::test]
    async fn send_email_timesout_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let (email_client, _, _) = email_client(format!("{}/v3/send", mock_server.uri()));

        // Delay by 3 minutes
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        let _ = assert_err!(outcome);
    }
}
