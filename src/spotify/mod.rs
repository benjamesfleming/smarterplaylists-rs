use rspotify;
use rspotify::Token;
use std::env;

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

// --

pub mod auth {

    use rspotify::prelude::*;
    use rspotify::Token;

    // Request the access/refresh token using the given auth code.
    // The return tokens should be persisted in the database
    pub fn request_token(code: &str) -> Result<Token, String> {
        let spotify = crate::spotify::init(None); // Init an unauthentication spotify client

        // Request the access/refresh tokens.
        // Note: This authenticates the current client instance, for future requests
        match spotify.request_token(&code) {
            Ok(_) => {
                // Get the tokens from the client, and return the to the
                // caller for storing in the db
                if let Ok(token) = spotify.get_token().lock() {
                    Ok(token.clone().unwrap())

                // Error - failed to get token??? shouldn't happen
                } else {
                    Err("Failed to acquire token lock".to_owned())
                }
            }
            // Error - failed request
            Err(err) => Err(format!("Failed to request token: {}", err)),
        }
    }

    // Build the authorize URL for the current client instance.
    // Note: This uses the scopes and callback URL defined in the init helper
    pub fn authorize_uri() -> String {
        crate::spotify::init(None).get_authorize_url(true).unwrap()
    }
}
