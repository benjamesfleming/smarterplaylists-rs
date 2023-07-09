use rspotify::model::UserId;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

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
