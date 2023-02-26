use crate::{error::*, macros, models::*, ApplicationState};
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use rspotify::{model::UserId, prelude::*};
use serde::Deserialize;

#[get("/auth/me")]
pub async fn auth_me_handler(
    session: Session,
    app: web::Data<ApplicationState>,
) -> Result<impl Responder, PublicError> {
    let user_id = macros::user_id!(session);
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE spotify_id = ?")
        .bind(user_id)
        .fetch_one(&app.db)
        .await?;

    Ok(HttpResponse::Ok().json(user))
}

//

#[get("/auth/spotify/sso")]
pub async fn auth_sso_redirect_handler() -> impl Responder {
    HttpResponse::TemporaryRedirect()
        .insert_header(("Location", crate::spotify::auth::authorize_uri()))
        .finish()
}

//

#[derive(Deserialize)]
pub struct AuthProviderCallbackParams {
    code: String,
}

#[get("/auth/spotify/callback")]
pub async fn auth_sso_callback_handler(
    session: Session,
    app: web::Data<ApplicationState>,
    params: web::Query<AuthProviderCallbackParams>,
) -> Result<impl Responder, PublicError> {
    let token = crate::spotify::auth::request_token(&params.code)?;
    let token_json = serde_json::to_string(&token)
        .map_err(|err| format!("Failed to serialize token to JSON: {}", err))?;

    // Request the user data
    let spotify_user = crate::spotify::init(Some(token)).me()?;

    // Check if we already know that user
    // If not, insert the initial database record
    let query = sqlx::query_as::<_, User>("SELECT * FROM users WHERE spotify_id = ?")
        .bind(&spotify_user.id.to_string())
        .fetch_optional(&app.db)
        .await?;

    match query {
        // We do know this user, just replace the access token
        Some(user) => {
            sqlx::query("UPDATE users SET spotify_access_token = ? WHERE spotify_id = ?")
                .bind(&token_json)
                .bind(&user.spotify_id)
                .execute(&app.db)
                .await?;
        }

        // We don't know this user
        None => {
            sqlx::query(
                "INSERT INTO users (spotify_id, spotify_username, spotify_email, spotify_access_token) VALUES (?, ?, ?, ?)"
            )
                .bind(&spotify_user.id.to_string())
                .bind(&spotify_user.display_name)
                .bind(&spotify_user.email)
                .bind(&token_json)
                .execute(&app.db)
                .await?;
        }
    };

    // Save the user id into the session cookie
    session.insert("user_id", spotify_user.id.to_string())?;

    // Redirect the user to the home page
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header(("Location", "/"))
        .finish())
}
