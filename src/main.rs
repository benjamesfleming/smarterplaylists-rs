mod assets;

use actix_web::{main, web, App, HttpServer};
use sqlx::sqlite::SqlitePool;

mod api {

    use crate::db::models::*;
    use crate::{assets, ApplicationState};

    use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

    #[get("/{path:.*}")]
    pub async fn index_get_handler(path: web::Path<String>) -> impl Responder {
        assets::to_http_response(&path)
    }

    // API Endpoints

    #[get("/api/v1/users/list")]
    pub async fn v1_users_list_handler(app: web::Data<ApplicationState>) -> impl Responder {
        let users = sqlx::query_as!(User, "SELECT * FROM users")
            .fetch_all(&app.db)
            .await;

        web::Json(users.unwrap())
    }

}

mod db {
    pub mod models {

        /// User holds the details of an authenticated spotify user
        #[derive(serde::Serialize)]
        pub struct User {
            pub id: Option<i64>,
            pub spotify_username: String,
            pub spotify_email: String,
            pub spotify_access_token: Option<String>,
            pub spotify_refresh_token: Option<String>,
        }
    }
}

pub struct ApplicationState {
    db: SqlitePool,
}

#[main]
async fn main() -> std::io::Result<()> {
    let pool = SqlitePool::connect("smarterplaylists-rs.db3")
        .await
        .unwrap();

    let state = web::Data::new(ApplicationState { db: pool });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(api::v1_users_list_handler)
            .service(api::index_get_handler)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
