use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token.")]
    InvalidToken,
    #[error("Missing token.")]
    MissingToken,
    #[error("Expired token.")]
    ExpiredToken,
    #[error("Invalid token format.")]
    InvalidTokenFormat,
    #[error("JWKS error")]
    JwksError,
    #[error("Invalid state")]
    InvalidState,
    #[error("Not Implemented")]
    NotImplemented,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing token"),
            AuthError::ExpiredToken => (StatusCode::UNAUTHORIZED, "Token has expired"),
            AuthError::InvalidTokenFormat => (StatusCode::UNAUTHORIZED, "Invalid token format"),
            AuthError::JwksError => (StatusCode::INTERNAL_SERVER_ERROR, "JWKS error"),
            AuthError::InvalidState => (StatusCode::INTERNAL_SERVER_ERROR, "Invalid state"),
            AuthError::NotImplemented => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Auth not implemented")
            }
        };

        (status, message).into_response()
    }
}
