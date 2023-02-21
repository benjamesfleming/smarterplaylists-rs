mod assets;
mod components;
mod spotify;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{main, web, App, HttpServer};
use sqlx::sqlite::SqlitePool;

mod api {

    use crate::db::models::*;
    use crate::{assets, ApplicationState};

    use actix_web::{get, web, HttpResponse, Responder};
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct AuthProviderCallbackParams {
        code: String,
    }

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

    // Auth Endpoints

    macro_rules! err_unknown_provider {
        ($provider: expr) => {
            HttpResponse::BadRequest()
                .body(format!("Unknown authentication provider: {}!", $provider))
        };
    }

    #[get("/auth/{provider}/sso")]
    pub async fn auth_provider_sso_redirect_handler(path: web::Path<String>) -> impl Responder {
        match path.as_str() {
            // Handle Spotify SSO redirect
            "spotify" => HttpResponse::TemporaryRedirect()
                .insert_header(("Location", crate::spotify::auth::authorize_uri()))
                .finish(),

            provider => err_unknown_provider!(provider),
        }
    }

    #[get("/auth/{provider}/callback")]
    pub async fn auth_provider_sso_callback_handler(
        path: web::Path<String>,
        params: web::Query<AuthProviderCallbackParams>,
    ) -> impl Responder {
        match path.as_str() {
            // Handle Spotify SSO Callback
            // TODO: Save credentials to the database, start the user session, and rediect back home
            "spotify" => {
                if let Ok(token) = crate::spotify::auth::request_token(&params.code) {
                    HttpResponse::Ok()
                        .body(format!("Access Token: {}", token.unwrap().access_token))
                } else {
                    HttpResponse::InternalServerError().finish()
                }
            }

            provider => err_unknown_provider!(provider),
        }
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

use dotenv::dotenv;

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
            .service(api::auth_provider_sso_redirect_handler)
            .service(api::auth_provider_sso_callback_handler)
            .service(api::index_get_handler)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
