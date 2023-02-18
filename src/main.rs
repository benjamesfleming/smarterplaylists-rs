mod assets;

use actix_web::{main, App, HttpServer};

mod api {

    use crate::assets;
    use actix_web::{get, web, Responder};

    #[get("/{path:.*}")]
    pub async fn index_get_handler(path: web::Path<String>) -> impl Responder {
        assets::to_http_response(&path)
    }
}

#[main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(api::index_get_handler))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
