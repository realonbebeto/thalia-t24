#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct StdResponse<'a> {
    pub message: &'a str,
}
