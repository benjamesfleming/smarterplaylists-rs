use actix_web::{get, web, Responder};

#[get("/api/v1/web/components/schema")]
pub async fn api_v1_web_components_schema() -> impl Responder {
    let schema = crate::components::Component::json_schema();
    web::Json(schema)
}
