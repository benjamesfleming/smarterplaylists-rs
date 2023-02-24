use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum PublicError {
    #[display(fmt = "An internal error occurred. Please try again later.")]
    InternalError,

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
                r#"{{"status": "{}", "message": "{}"}}"#,
                self.status_code(),
                self
            ))
    }

    // Map the error to an HTTP status code
    fn status_code(&self) -> StatusCode {
        match *self {
            PublicError::Unauthorized => StatusCode::UNAUTHORIZED, // 401
            PublicError::InternalError => StatusCode::INTERNAL_SERVER_ERROR, // 500
        }
    }
}

impl From<sqlx::Error> for PublicError {
    fn from(_: sqlx::Error) -> Self {
        Self::InternalError
    }
}
