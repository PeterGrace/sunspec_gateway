use crate::modules::users::User;
use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;
use std::error::Error;
use utoipa::ToSchema;

pub mod users;

#[derive(PartialEq)]
pub enum AuthorizableType {
    User(User),
}

#[derive(PartialEq)]
pub enum RBAC {
    Read,
    Write,
    Delete,
    Admin,
}

#[async_trait]
pub trait Authorizable {
    async fn check_authorization<'a>(
        id: &'a AuthorizableType,
        user: &'a User,
        rbac: &'a RBAC,
    ) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(crate) struct AppAPIResponse {
    /// A string message explaining the response
    message: String,
    /// optional json-formatted data related to the response
    data: Option<Value>,
}
impl AppAPIResponse {
    pub fn message<S: Into<String>>(msg: S) -> Self {
        Self {
            message: msg.into(),
            data: None,
        }
    }
    pub fn data<S: Into<String>, D: Into<Value>>(msg: S, data: D) -> Self {
        Self {
            message: msg.into(),
            data: Some(data.into()),
        }
    }
}
impl IntoResponse for AppAPIResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
