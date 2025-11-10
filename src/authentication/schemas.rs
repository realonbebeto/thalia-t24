#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum LoginIdentifier {
    Email(String),
    Username(String),
}

#[derive(serde::Deserialize, utoipa::ToSchema, Debug)]
pub struct LoginRequest {
    pub login_id: LoginIdentifier,
    pub password: String,
}
