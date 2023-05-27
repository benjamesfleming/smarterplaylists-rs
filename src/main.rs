mod assets;
mod cache;
mod controller;
mod components;
mod error;
mod handlers;
mod macros;
mod models;
mod routes;
mod spotify;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    http::StatusCode,
    main,
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web, App, HttpServer,
};
use cache::RedisPool;
use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;

pub struct ApplicationState {
    db: SqlitePool,
    cache: RedisPool
}

#[main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "warn");
    std::env::set_var("RUST_BACKTRACE", "0");
    env_logger::init();

    // SQLite DB Connection Pool
    let db_pool = SqlitePool::connect("smarterplaylists-rs.db3?mode=rwc")
        .await
        .unwrap();

    // Run SQLx migrations -
    // These are all embeded into the binary at build time
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .unwrap();

    // Redis Cache Pool
    let cache_pool = cache::connect().await.unwrap();

    // Application Session Management
    // TODO: Pull session key from environment variable
    let session_key = Key::from(
        b"N4yGxwsXHqY0r2p5hLSmrwFdTEhY9KSwt4byWzFvuK25dNu/fs460VEOukuwoD5M5qvN94aDXtYolImdfCBETQ==",
    );

    // Application State
    let state = web::Data::new(ApplicationState { 
        db: db_pool,
        cache: cache_pool
    });

    // --

    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                session_key.clone(),
            ))
            .wrap(ErrorHandlers::new().handler(StatusCode::INTERNAL_SERVER_ERROR, error_logger))
            .app_data(state.clone())
            .service(routes::router())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

//

fn error_logger<B>(
    res: actix_web::dev::ServiceResponse<B>,
) -> actix_web::Result<actix_web::middleware::ErrorHandlerResponse<B>> {
    log::error!("{:?}", res.response().error().unwrap());

    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
}
