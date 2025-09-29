#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct StdResponse<'a> {
    message: &'a str,
}

impl<'a> StdResponse<'a> {
    pub fn from(message: &'a str) -> Self {
        StdResponse { message }
    }
}
