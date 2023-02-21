mod assets;

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

mod spotify {

    use rspotify;
    use rspotify::Token;
    use std::env;

    pub mod auth {

        use rspotify::prelude::*;
        use rspotify::Token;

        // Request the access/refresh token using the given auth code.
        // The return tokens should be persisted in the database
        pub fn request_token(code: &str) -> Result<Option<Token>, String> {
            let spotify = crate::spotify::init(None); // Init an unauthentication spotify client

            // Request the access/refresh tokens.
            // Note: This authenticates the current client instance, for future requests
            if let Ok(_) = spotify.request_token(&code) {
                // Get the tokens from the client, and return the to the
                // caller for storing in the db
                if let Ok(token) = spotify.get_token().lock() {
                    Ok(token.clone())

                // Error - failed to get token??? shouldn't happen
                } else {
                    Err("Failed to acquire token lock".into())
                }

            // Error - failed request
            } else {
                Err("Failed to request token".into())
            }
        }

        // Build the authorize URL for the current client instance.
        // Note: This uses the scopes and callback URL defined in the init helper
        pub fn authorize_uri() -> String {
            crate::spotify::init(None).get_authorize_url(true).unwrap()
        }
    }

    pub fn init(token: Option<Token>) -> rspotify::AuthCodeSpotify {
        // RSpotify Instance
        // Note: Pull OAuth client id/client secret from environment variables, panicing if not found
        let spotify_creds = rspotify::Credentials::new(
            &env::var("SPL_SPOTIFY_CLIENT_ID").expect("$SPL_SPOTIFY_CLIENT_ID is not set"),
            &env::var("SPL_SPOTIFY_CLIENT_SECRET").expect("$SPL_SPOTIFY_CLIENT_SECRET is not set"),
        );

        let spotify_oauth = rspotify::OAuth {
            // Scopes - Add scopes for reading and writing to a users playlists
            // @ref https://developer.spotify.com/documentation/general/guides/authorization/scopes
            scopes: rspotify::scopes!(
                "playlist-read-private",   // Read access to user's private playlists.
                "playlist-modify-private", // Write access to a user's private playlists.
                "playlist-modify-public",  // Write access to a user's public playlists.
                "user-follow-read", // Read access to the list of artists and other users that the user follows.
                "user-read-email",  // Read access to user’s email address.
                "user-library-read"  // Read access to a user's library.
            ),

            // Redirect URI
            // TODO: Dynamicly build this based on production/public URL environment variable
            redirect_uri: "http://127.0.0.1:8080/auth/spotify/callback".to_owned(),
            ..Default::default()
        };

        let spotify = rspotify::AuthCodeSpotify::new(spotify_creds, spotify_oauth);

        // If an access token was provided, then add it to the Spotify API client
        // Note: If not provided, the APIs that request authentication will fail
        if let Some(token) = token {
            *spotify.token.lock().unwrap() = Some(token)
        }

        spotify
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
