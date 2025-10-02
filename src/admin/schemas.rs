use uuid::Uuid;

#[derive(Debug, utoipa::ToSchema, serde::Deserialize)]
pub struct ChartAccountRequest {
    pub name: String,
    pub code: String,
    pub coa_type: String,
    pub currency: String,
}

#[derive(Debug, utoipa::ToSchema, serde::Deserialize)]
pub struct AccountTypeRequest {
    pub name: String,
    pub coa_id: Uuid,
    pub description: String,
}
