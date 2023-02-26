use crate::assets;
use actix_web::{get, web, Responder, Scope};
use std::io;

pub fn router() -> Scope {
    web::scope("")
        // API Routes
        // Auth Routes
        .service(crate::handlers::auth::auth_me_handler)
        .service(crate::handlers::auth::auth_sso_redirect_handler)
        .service(crate::handlers::auth::auth_sso_callback_handler)
        // --
        .service(index_get_handler)
}

//

#[get("/{path:.*}")]
pub async fn index_get_handler(path: web::Path<String>) -> io::Result<impl Responder> {
    Ok(assets::to_http_response(&path))
}
