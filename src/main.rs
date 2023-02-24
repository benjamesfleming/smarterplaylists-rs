mod assets;
mod components;
mod error;
mod handlers;
mod macros;
mod models;
mod routes;
mod spotify;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, main, web, App, HttpServer};
use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;

pub struct ApplicationState {
    db: SqlitePool,
}

#[main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // SQLite DB Connection Pool
    let pool = SqlitePool::connect("smarterplaylists-rs.db3")
        .await
        .unwrap();

    // Application Session Management
    // TODO: Pull session key from environment variable
    let session_key = Key::generate();

    // Application State
    let state = web::Data::new(ApplicationState { db: pool });

    // --

    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                session_key.clone(),
            ))
            .app_data(state.clone())
            .service(routes::router())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
