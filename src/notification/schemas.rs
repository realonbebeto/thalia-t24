use askama::Template;

#[derive(serde::Serialize)]
pub struct Recipient<'a> {
    email: &'a str,
}

impl<'a> Recipient<'a> {
    pub fn new(email: &'a str) -> Self {
        Self { email }
    }
}

#[derive(serde::Serialize)]
pub struct SendEmailRequest<'a> {
    #[serde(rename = "FromEmail")]
    fromemail: &'a str,
    #[serde(rename = "FromName")]
    fromname: &'a str,
    #[serde(rename = "Subject")]
    subject: &'a str,
    #[serde(rename = "Text-part")]
    text_part: &'a str,
    #[serde(rename = "Html-part")]
    html_part: &'a str,
    #[serde(rename = "Recipients")]
    recipients: Vec<Recipient<'a>>,
}

impl<'a> SendEmailRequest<'a> {
    pub fn new(
        fromemail: &'a str,
        fromname: &'a str,
        subject: &'a str,
        text_part: &'a str,
        html_part: &'a str,
        recipients: Vec<Recipient<'a>>,
    ) -> Self {
        Self {
            fromemail,
            fromname,
            subject,
            text_part,
            html_part,
            recipients,
        }
    }
}

#[derive(Template)]
#[template(path = "welcome_email.html")]
pub struct WelcomeEmailTemplate<'a> {
    first_name: &'a str,
    confirmation_link: &'a str,
    company_name: &'a str,
}

impl<'a> WelcomeEmailTemplate<'a> {
    pub fn new(first_name: &'a str, confirmation_link: &'a str, company_name: &'a str) -> Self {
        Self {
            first_name,
            confirmation_link,
            company_name,
        }
    }
}

#[derive(Template)]
#[template(path = "welcome_email.html")]
pub struct WelcomeEmailTemplateTxt<'a> {
    first_name: &'a str,
    confirmation_link: &'a str,
    company_name: &'a str,
}

impl<'a> WelcomeEmailTemplateTxt<'a> {
    pub fn new(first_name: &'a str, confirmation_link: &'a str, company_name: &'a str) -> Self {
        Self {
            first_name,
            confirmation_link,
            company_name,
        }
    }
}
