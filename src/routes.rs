use crate::assets;
use actix_web::{get, web, Responder, Scope};
use std::io;

pub fn router() -> Scope {
    web::scope("")
        // API Routes
        .service(crate::handlers::api_spotify::api_v1_spotify_user_playlists)
        // Auth Routes
        .service(crate::handlers::auth::auth_me_handler)
        .service(crate::handlers::auth::auth_sso_redirect_handler)
        .service(crate::handlers::auth::auth_sso_callback_handler)
        // Web Routes
        .service(crate::handlers::api_web::api_v1_web_components_schema)
        // --
        .service(index_get_handler)
}

//

#[get("/{path:.*}")]
pub async fn index_get_handler(path: web::Path<String>) -> io::Result<impl Responder> {
    Ok(assets::to_http_response(&path))
}
