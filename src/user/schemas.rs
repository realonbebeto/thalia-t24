use sqlx::types::chrono;
use uuid::Uuid;
#[derive(Debug, utoipa::ToSchema, serde::Deserialize)]
pub struct UserCreateRequest {
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: chrono::NaiveDate,
    pub username: String,
    pub password: String,
    pub email: String,
    pub access_role: String,
}

#[derive(sqlx::FromRow)]
#[allow(unused)]
pub struct UserResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub access_role: String,
}
