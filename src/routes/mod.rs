pub mod pokemon;

use actix_web::HttpResponse;

pub async fn healthz() -> HttpResponse {
    HttpResponse::Ok().finish()
}
