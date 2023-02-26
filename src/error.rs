use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum PublicError {
    #[display(fmt = "An internal error occurred. Please try again later.")]
    InternalError { inner: Box<dyn std::error::Error> },
    #[display(fmt = "Unauthorized. You are not allowed to access that resource.")]
    Unauthorized,
}

impl actix_web::error::ResponseError for PublicError {
    /// Override the default HTML response to return
    /// a JSON object.
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(format!(
                r#"{{"status": "error", "code": {}, "message": "{}"}}"#,
                self.status_code().as_u16(),
                self
            ))
    }

    // Map the error to an HTTP status code
    fn status_code(&self) -> StatusCode {
        match *self {
            PublicError::Unauthorized => StatusCode::UNAUTHORIZED, // 401
            PublicError::InternalError { inner: _ } => StatusCode::INTERNAL_SERVER_ERROR, // 500
        }
    }
}

//

macro_rules! map_internal_error {
    ($($x: ty),+ $(,)?) => {
        $(
            impl From<$x> for PublicError {
                fn from(inner: $x) -> Self {
                    Self::InternalError {
                        inner: Box::from(inner),
                    }
                }
            }
        )+
    };
}

map_internal_error![
    actix_session::SessionGetError,
    actix_session::SessionInsertError,
    rspotify::ClientError,
    sqlx::Error,
    // Map string types to internal error
    // USAGE:
    //     call_will_fail().map_err(|_| "Oh no! This call has failed")?
    &'_ str,
    String,
];
