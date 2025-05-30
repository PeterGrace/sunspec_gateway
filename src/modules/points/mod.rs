use crate::consts::*;
use crate::modules::users::User;
use crate::modules::{AppAPIResponse, Authorizable, AuthorizableType, RBAC};
use crate::routes::USERS_TAG;
use crate::state::AppState;
use async_trait::async_trait;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{debug_handler, Json};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::error::Error;
use tower_sessions::Session;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub(crate) fn point_routes(state: AppState) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_point))
        .with_state(state)
}
pub struct Point;
#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct PointResponse {
    pub model: i32,
    pub name: String,
    pub description: String,
}
#[async_trait]
impl Authorizable for Point {
    async fn check_authorization<'a>(
        id: &'a AuthorizableType,
        user: &'a User,
        rbac: &'a RBAC,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // eventually I may want to define access levels but it's not needed right now
        Ok(true)
    }
}
#[debug_handler]
#[utoipa::path(
get,
path = "/{:model}/{:point}",
summary = "retrieve a specific point details",
params(
("model" = i32, Path, description = "Model number for point"),
("point" = String, Path, description = "Name of point"),
),
responses(
(status = NO_CONTENT, description = "successful request"),
(status = BAD_REQUEST, description = "bad request", body = AppAPIResponse)),
tag = POINTS_TAG
)]
pub async fn get_point(
    State(state): State<AppState>,
    session: Session,
    Path((model, point)): Path<(i32, String)>,
) -> Result<Json<PointResponse>, (StatusCode, AppAPIResponse)> {
    let user: User = match session.get("user").await {
        Ok(session_request) => match session_request {
            Some(user) => user,
            None => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppAPIResponse::message("Couldn't get user id from session"),
                ));
            }
        },
        Err(e) => {
            error!("Error getting user from session: {e}");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppAPIResponse::message("Couldn't get user id from session"),
            ));
        }
    };
    if let Ok(authorized) = user
        .is_authorized::<Point>(
            &AuthorizableType::User(User {
                id: model,
                ..User::default()
            }),
            &RBAC::Delete,
        )
        .await
    {
        if !authorized {
            return Err((
                StatusCode::FORBIDDEN,
                AppAPIResponse::message("You are not authorized to this action."),
            ));
        } else {
            let pr = PointResponse {
                model,
                name: point,
                description: "".to_string(),
            };
            Ok(Json(pr))
        }
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppAPIResponse::message("Could not check user authorizations"),
        ));
    }
}
