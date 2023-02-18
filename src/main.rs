mod assets;

use actix_web::{main, App, HttpServer};

mod api {

    use crate::assets;
    use actix_web::{get, Responder};

    #[get("/")]
    pub async fn index_get_handler() -> impl Responder {
        assets::to_http_response("index.html")
    }
}

#[main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(api::index_get_handler))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
