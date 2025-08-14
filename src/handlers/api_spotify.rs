use actix_web::{get, web, Responder};
use rspotify::{model::SimplifiedPlaylist, prelude::*};

use crate::{cache, error::PublicError, models::User, spotify, ApplicationState};

#[get("/api/v1/spotify/user_playlists")]
pub async fn api_v1_spotify_user_playlists(
    user: User,
    app: web::Data<ApplicationState>,
) -> Result<impl Responder, PublicError> {
    let key = format!("user_playlists:{}", user.id());
    let res = cache::get_or_create(&app.cache, key.as_str(), 300, false, || {
        let mut playlists: Vec<SimplifiedPlaylist> = Vec::new();
        for plst in spotify::init(user.token()).user_playlists(user.spotify_id()) {
            playlists.push(plst?);
        }
        Ok(playlists)
    })
    .await?;

    Ok(web::Json(res))
}
