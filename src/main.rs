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

    use actix_session::Session;
    use actix_web::{get, web, HttpResponse, HttpResponseBuilder, Responder};
    use rspotify::prelude::*;
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

    // Auth Endpoints

    #[get("/auth/me")]
    pub async fn auth_me_handler(
        session: Session,
        app: web::Data<ApplicationState>,
    ) -> impl Responder {
        let user_id = match session.get::<i64>("user_id") {
            Ok(Some(id)) => id,
            Ok(None) => {
                return HttpResponse::Unauthorized().finish();
            }
            Err(_) => {
                return HttpResponse::InternalServerError().finish();
            }
        };
        let user = sqlx::query!("SELECT * FROM users WHERE id = ?", user_id)
            .map(|row| {
                let token: rspotify::Token =
                    serde_json::from_str(&row.spotify_access_token.unwrap()).unwrap();
                User {
                    id: row.id,
                    spotify_username: row.spotify_username,
                    spotify_email: row.spotify_email,
                    spotify_access_token: Some(token),
                }
            })
            .fetch_one(&app.db)
            .await
            .unwrap();

        HttpResponse::Ok().json(user)
    }

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
        session: Session,
        app: web::Data<ApplicationState>,
        path: web::Path<String>,
        params: web::Query<AuthProviderCallbackParams>,
    ) -> impl Responder {
        match path.as_str() {
            // Handle Spotify SSO Callback
            // TODO: Save credentials to the database, start the user session, and rediect back home
            "spotify" => {
                // Request the access/secret token
                if let Ok(Some(token)) = crate::spotify::auth::request_token(&params.code) {
                    // Request the user data
                    if let Ok(user) = crate::spotify::init(Some(token.clone())).me() {
                        let token_json = serde_json::to_string(&token).unwrap();
                        // Check if we already know that user
                        // If not, insert the initial database record
                        let query = sqlx::query!(
                            "SELECT id FROM users WHERE spotify_email = ?",
                            user.email
                        );
                        let user_id: i64 = match query.fetch_optional(&app.db).await {
                            // We do know this user, just replace the access token
                            Ok(Some(user)) => {
                                let query = sqlx::query!(
                                    "UPDATE users SET spotify_access_token = ? WHERE id = ?",
                                    token_json,
                                    user.id
                                );
                                // Update access token into the database
                                match query.execute(&app.db).await {
                                    Ok(_) => user.id,
                                    Err(_) => return HttpResponse::InternalServerError().finish(),
                                }
                            }
                            // We don't know this user
                            Ok(None) => {
                                let query =
                                    sqlx::query!(
                                        "INSERT INTO users (spotify_username, spotify_email, spotify_access_token) VALUES (?, ?, ?)",
                                        user.display_name,
                                        user.email,
                                        token_json
                                    );
                                // Insert user data into the database
                                match query.execute(&app.db).await {
                                    Ok(res) => res.last_insert_rowid(),
                                    Err(_) => return HttpResponse::InternalServerError().finish(),
                                }
                            }
                            // Query Failed
                            Err(_) => return HttpResponse::InternalServerError().finish(),
                        };
                        // Save the user id into the session cookie
                        if let Ok(_) = session.insert("user_id", user_id) {
                            // Redirect the user to the home page
                            return HttpResponse::TemporaryRedirect()
                                .insert_header(("Location", "/"))
                                .finish();
                        }
                    }
                }
                return HttpResponse::InternalServerError().finish();
            }

            provider => err_unknown_provider!(provider),
        }
    }
}

mod db {
    pub mod models {
        use rspotify::Token;
        use serde::{Deserialize, Serialize};

        /// User holds the details of an authenticated spotify user
        #[derive(Serialize, Deserialize)]
        pub struct User {
            pub id: i64,
            pub spotify_username: String,
            pub spotify_email: String,
            pub spotify_access_token: Option<Token>,
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
            .service(api::auth_me_handler)
            .service(api::index_get_handler)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
