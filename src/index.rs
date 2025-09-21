use actix_web::HttpResponse;

use crate::base::StdResponse;

#[utoipa::path(get, path = "/home/index")]
pub async fn index_page() -> &'static str {
    "Thalia - T24"
}

#[utoipa::path(get, path="/home/health", responses((status=200, description="Health status")))]
pub async fn health_check() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(StdResponse {
        message: "Up and running",
    }))
}
