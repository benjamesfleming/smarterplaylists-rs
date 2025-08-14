use actix_session::SessionExt;
use actix_web::{web, HttpRequest};
use futures_util::future::LocalBoxFuture;
use rspotify::model::UserId;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{error::PublicError, ApplicationState};

/// User holds the details of an authenticated spotify user.
///
/// The most up-to-date spotify token is stored in the `spotify_access_token` row as a JSON string.
/// We impl a custom From/Into for the access token to allow for this behaviour.
#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub spotify_id: String,
    pub spotify_username: String,
    pub spotify_email: String,
    #[sqlx(default, try_from = "String")]
    pub spotify_access_token: Token,
}

impl User {
    pub fn id(&self) -> Ulid {
        Ulid::from_string(&self.id).unwrap()
    }

    pub fn spotify_id(&self) -> UserId<'_> {
        UserId::from_uri(self.spotify_id.as_str()).unwrap()
    }

    pub fn token(&self) -> Option<rspotify::Token> {
        Some(self.spotify_access_token.0.to_owned().unwrap())
    }
}

impl actix_web::FromRequest for User {
    type Error = PublicError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        // Extract app state or return an internal error
        let app = match req.app_data::<web::Data<ApplicationState>>() {
            Some(data) => data.clone(),
            None => {
                return Box::pin(async { Err(PublicError::from("Application state not found")) });
            }
        };

        // Extract user_id from session or return the an error
        let session_result = req.get_session().get::<String>("user_id");
        let user_id = match session_result {
            Ok(Some(id)) => id,
            Ok(None) => {
                return Box::pin(async { Err(PublicError::Unauthorized) });
            }
            Err(e) => {
                return Box::pin(async { Err(PublicError::from(e)) });
            }
        };

        Box::pin(async move {
            let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_one(&app.db)
                .await?;
            Ok(user)
        })
    }
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
