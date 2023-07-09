use actix_session::Session;
use actix_web::{get, web, Responder};
use rspotify::{model::SimplifiedPlaylist, prelude::*};

use crate::{cache, error::PublicError, macros, models::User, spotify, ApplicationState};

#[get("/api/v1/spotify/user_playlists")]
pub async fn api_v1_spotify_user_playlists(
    session: Session,
    app: web::Data<ApplicationState>,
) -> Result<impl Responder, PublicError> {
    let user_id = macros::user_id!(session);
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_one(&app.db)
        .await?;

    let key = format!("user_playlists:{user_id}");
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
