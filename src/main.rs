mod assets;
mod components;
mod spotify;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{main, web, App, HttpServer};
use sqlx::sqlite::SqlitePool;

mod api {

    use crate::db::models::*;
    use crate::{assets, macros, ApplicationState};

    use actix_session::Session;
    use actix_web::http::header::ContentType;
    use actix_web::http::StatusCode;
    use actix_web::{error, get, web, HttpResponse, Responder};
    use derive_more::{Display, Error};
    use rspotify::prelude::*;
    use serde::Deserialize;
    use std::io;

    // Errors

    #[derive(Debug, Display, Error)]
    pub enum PublicError {
        #[display(fmt = "An internal error occurred. Please try again later.")]
        InternalError,

        #[display(fmt = "Unauthorized. You are not allowed to access that resource.")]
        Unauthorized,

        #[display(fmt = "Unknown authentication provider.")]
        UnknownProvider,
    }

    impl error::ResponseError for PublicError {
        fn error_response(&self) -> HttpResponse {
            HttpResponse::build(self.status_code())
                .insert_header(ContentType::json())
                .body(format!(
                    r#"{{"status": "{}", "message": "{}"}}"#,
                    self.status_code(),
                    self
                ))
        }

        fn status_code(&self) -> StatusCode {
            match *self {
                PublicError::UnknownProvider => StatusCode::BAD_REQUEST, // 400
                PublicError::Unauthorized => StatusCode::UNAUTHORIZED,   // 401
                PublicError::InternalError => StatusCode::INTERNAL_SERVER_ERROR, // 500
            }
        }
    }

    impl From<sqlx::Error> for PublicError {
        fn from(_: sqlx::Error) -> Self {
            Self::InternalError
        }
    }

    //

    #[derive(Deserialize)]
    pub struct AuthProviderCallbackParams {
        code: String,
    }

    #[get("/{path:.*}")]
    pub async fn index_get_handler(path: web::Path<String>) -> io::Result<impl Responder> {
        Ok(assets::to_http_response(&path))
    }

    // API Endpoints

    // Auth Endpoints

    #[get("/auth/me")]
    pub async fn auth_me_handler(
        session: Session,
        app: web::Data<ApplicationState>,
    ) -> Result<impl Responder, PublicError> {
        let user_id = macros::user_id!(session);
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&app.db)
            .await?;

        Ok(HttpResponse::Ok().json(user))
    }

    #[get("/auth/{provider}/sso")]
    pub async fn auth_provider_sso_redirect_handler(
        path: web::Path<String>,
    ) -> Result<impl Responder, PublicError> {
        let location = match path.as_str() {
            "spotify" => Ok(crate::spotify::auth::authorize_uri()),
            _ => Err(PublicError::UnknownProvider),
        }?;

        let res = HttpResponse::TemporaryRedirect()
            .insert_header(("Location", location))
            .finish();

        Ok(res)
    }

    #[get("/auth/{provider}/callback")]
    pub async fn auth_provider_sso_callback_handler(
        session: Session,
        app: web::Data<ApplicationState>,
        path: web::Path<String>,
        params: web::Query<AuthProviderCallbackParams>,
    ) -> Result<impl Responder, PublicError> {
        match path.as_str() {
            "spotify" => {
                let token = crate::spotify::auth::request_token(&params.code)
                    .map_err(|_| PublicError::InternalError)?
                    .ok_or(PublicError::InternalError)?;

                let token_json =
                    serde_json::to_string(&token).map_err(|_| PublicError::InternalError)?;

                // Request the user data
                let spotify_user = crate::spotify::init(Some(token))
                    .me()
                    .map_err(|_| PublicError::InternalError)?;

                // Check if we already know that user
                // If not, insert the initial database record
                let query = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
                    .bind(&spotify_user.email)
                    .fetch_optional(&app.db)
                    .await?;

                let user_id: i64 = match query {
                    // We do know this user, just replace the access token
                    Some(user) => {
                        sqlx::query("UPDATE users SET spotify_access_token = ? WHERE id = ?")
                            .bind(&token_json)
                            .bind(&user.id)
                            .execute(&app.db)
                            .await?;

                        user.id
                    }

                    // We don't know this user
                    None => {
                        sqlx::query(
                            "INSERT INTO users (spotify_username, spotify_email, spotify_access_token) VALUES (?, ?, ?)"
                        )
                            .bind(&spotify_user.display_name)
                            .bind(&spotify_user.email)
                            .bind(&token_json)
                            .execute(&app.db)
                            .await?
                            .last_insert_rowid()
                    }
                };

                // Save the user id into the session cookie
                session
                    .insert("user_id", user_id)
                    .map_err(|_| PublicError::InternalError)?;

                // Redirect the user to the home page
                Ok(HttpResponse::TemporaryRedirect()
                    .insert_header(("Location", "/"))
                    .finish())
            }

            _ => Err(PublicError::UnknownProvider),
        }
    }
}

mod macros {
    /// Extract the current user_id from the session.
    ///
    /// Returns:
    /// - PublicError::InternalError on failure, or
    /// - PublicError::Unauthorized if the user id wasn't found in the session.
    macro_rules! user_id {
        ($session: expr) => {
            $session
                .get::<i64>("user_id")
                .map_err(|_| PublicError::InternalError)? // Internal Error - failed to get session value
                .ok_or(PublicError::Unauthorized)? // Session key empty, user is unauthenticated
        };
    }

    pub(crate) use user_id;
}

mod db {
    pub mod models {
        use serde::{Deserialize, Serialize};

        /// User holds the details of an authenticated spotify user
        #[derive(sqlx::FromRow, Serialize, Deserialize)]
        pub struct User {
            pub id: i64,
            pub spotify_username: String,
            pub spotify_email: String,
            #[sqlx(default, try_from = "String")]
            pub spotify_access_token: Token,
        }

        /// Token holds the spotify auth details
        #[derive(Serialize, Deserialize)]
        pub struct Token(Option<rspotify::Token>);

        impl Default for Token {
            fn default() -> Self {
                Token(Some(rspotify::Token::default()))
            }
        }

        impl From<String> for Token {
            fn from(value: String) -> Self {
                serde_json::from_str(value.as_str()).unwrap()
            }
        }

        impl Into<String> for Token {
            fn into(self) -> String {
                serde_json::to_string(&self.0).unwrap()
            }
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
