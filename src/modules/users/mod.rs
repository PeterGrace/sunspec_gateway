use crate::consts::*;
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

pub(crate) fn user_routes(state: AppState) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create_user))
        .routes(routes!(get_user))
        .routes(routes!(get_users))
        .routes(routes!(update_user))
        .routes(routes!(delete_user))
        .with_state(state)
}

#[async_trait]
impl Authorizable for User {
    async fn check_authorization<'a>(
        id: &'a AuthorizableType,
        user: &'a User,
        rbac: &'a RBAC,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if let AuthorizableType::User(req) = id {
            let id = &req.id;
            if *id == 0 && *rbac == RBAC::Read {
                // probably need admin to list all users
                return Ok(true);
            }
            if *id == 0 && *rbac == RBAC::Write {
                // probably need admin to create users
                return Ok(true);
            }

            //TODO: Implement user database
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "invalid assertion: did not get proper AuthorizableType::User object, this is a code error"
            )));
            // let rs = sqlx::query_as!(
            //     User,
            //     "SELECT * FROM users where id = $1 and team_id = $2",
            //     id,
            //     user.team_id
            // )
            //     .fetch_one(pool)
            //     .await?;
            // Ok(if rs.id == *id {
            //     true
            // } else {
            //     false
            // })
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "invalid assertion: did not get proper AuthorizableType::User object, this is a code error"
            )))
        }
    }
}

//region get
#[debug_handler]
#[utoipa::path(
get,
path = "/",
summary = "get all users",
responses(
(status = OK, description = "successful request", body = Vec<User>),
(status = BAD_REQUEST, description = "bad request", body = AppAPIResponse)),
tag = USERS_TAG
)]
pub async fn get_users(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<Vec<User>>, (StatusCode, AppAPIResponse)> {
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
        .is_authorized::<User>(&AuthorizableType::User(User::default()), &RBAC::Read)
        .await
    {
        if !authorized {
            return Err((
                StatusCode::FORBIDDEN,
                AppAPIResponse::message("You are not authorized to this action."),
            ));
        }
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppAPIResponse::message("Could not check user authorizations"),
        ));
    }
    //TODO: Implement database for auth
    return Err((
        StatusCode::NOT_IMPLEMENTED,
        AppAPIResponse::message("auth not implemented"),
    ));

    // let rs = sqlx::query_as!(
    //     User,
    //     "SELECT * FROM users where team_id = $1",
    //     user.team_id
    // )
    //     .fetch_all(&state.pool.clone())
    //     .await;
    // if let Ok(data) = rs {
    //     return Ok(Json(data));
    // } else {
    //     return Ok(Json(vec![]));
    // }
}
#[debug_handler]
#[utoipa::path(
get,
path = "/{:id}",
summary = "get a specific user",
params(
("id" = i32, Path, description = "id for the user to retrieve")
),
responses(
(status = OK, description = "successful request", body = User),
(status = BAD_REQUEST, description = "bad request", body = AppAPIResponse)),
tag = USERS_TAG
)]
pub async fn get_user(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i32>,
) -> Result<Json<User>, (StatusCode, AppAPIResponse)> {
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
        .is_authorized::<User>(
            &AuthorizableType::User(User {
                id: id,
                ..User::default()
            }),
            &RBAC::Read,
        )
        .await
    {
        if !authorized {
            return Err((
                StatusCode::FORBIDDEN,
                AppAPIResponse::message("You are not authorized to this action."),
            ));
        }
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppAPIResponse::message("Could not check user authorizations"),
        ));
    }
    //TODO: Implement database for auth
    return Err((
        StatusCode::NOT_IMPLEMENTED,
        AppAPIResponse::message("auth not implemented"),
    ));

    // let rs = sqlx::query_as!(
    //     User,
    //     "SELECT * FROM users where id = $1 and team_id = $2",
    //     id,
    //     user.team_id
    // )
    //     .fetch_one(&state.pool.clone())
    //     .await;
    // if let Ok(data) = rs {
    //     return Ok(Json(data));
    // } else {
    //     return Err((StatusCode::NOT_FOUND, AppAPIResponse::message("user not found")));
    // }
}
//endregion

#[debug_handler]
#[utoipa::path(path="/",
post,
summary = "create new user record",
request_body(content_type = "application/json", content = CreateUser),
responses(
(status = CREATED, description = "successful request", body = AppAPIResponse),
(status = BAD_REQUEST, description = "bad request", body = AppAPIResponse)),
tag = USERS_TAG)]
pub async fn create_user(
    State(_state): State<AppState>,
    _session: Session,
    Json(_body): Json<CreateUser>,
) -> Result<String, (StatusCode, AppAPIResponse)> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        AppAPIResponse::message("Creating users is not yet implemented in the api"),
    ))
}

/// A User object
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, PartialEq, Default, ToSchema)]
pub(crate) struct User {
    /// The user's integer id
    pub(crate) id: i32,
    /// Commonly 'first name' in USA
    pub(crate) given_name: Option<String>,
    /// Commonly 'last name' in USA
    pub(crate) family_name: Option<String>,
    /// Middle name in USA, for i18n purposes, extra name values
    pub(crate) additional_name: Option<String>,
    /// User's preferred display name
    pub(crate) preferred_name: Option<String>,
    /// the login associated with the user
    pub(crate) login: String,
    /// the team the user is assoociated with
    pub(crate) team_id: i32,
}
impl User {
    pub async fn is_authorized<T: Authorizable>(
        &self,
        id: &AuthorizableType,
        rbac: &RBAC,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(T::check_authorization(&id, &self, rbac).await?)
    }
}

/// A payload for creating/updating a user
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, PartialEq, Default, ToSchema)]
pub struct CreateUser {
    /// Commonly 'first name' in USA
    pub(crate) given_name: Option<String>,
    /// Commonly 'last name' in USA
    pub(crate) family_name: Option<String>,
    /// Middle name in USA, for i18n purposes, extra name values
    pub(crate) additional_name: Option<String>,
    /// User's preferred display name
    pub(crate) preferred_name: Option<String>,
}

//region update
#[debug_handler]
#[utoipa::path(
put,
path = "/{:id}",
summary = "update a user record",
params(
("id" = i32, Path, description = "id for the user to update")
),
request_body(content_type = "application/json", content = CreateUser),
responses(
(status = NO_CONTENT, description = "successful request", body = AppAPIResponse),
(status = BAD_REQUEST, description = "bad request", body = AppAPIResponse)),
tag = USERS_TAG
)]
pub async fn update_user(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i32>,
    Json(payload): Json<CreateUser>,
) -> Result<AppAPIResponse, (StatusCode, AppAPIResponse)> {
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
        .is_authorized::<User>(
            &AuthorizableType::User(User {
                id: id,
                ..User::default()
            }),
            &RBAC::Write,
        )
        .await
    {
        if !authorized {
            return Err((
                StatusCode::FORBIDDEN,
                AppAPIResponse::message("You are not authorized to this action."),
            ));
        }
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppAPIResponse::message("Could not check user authorizations"),
        ));
    }
    // TODO: figure out how the hell to dynamically build the query to update all the user fields

    //TODO: Implement database for auth
    return Err((
        StatusCode::NOT_IMPLEMENTED,
        AppAPIResponse::message("auth not implemented"),
    ));

    // match sqlx::query!(
    //     "UPDATE users set given_name = $1, family_name = $2 where id = $3",
    //     payload.given_name.unwrap_or_default(),
    //     payload.family_name.unwrap_or_default(),
    //     user.id
    // )
    //     .execute(&state.pool.clone())
    //     .await
    // {
    //     Ok(_) => {
    //         Ok(AppAPIResponse::message("User updated."))
    //     }
    //     Err(e) => Err((
    //         StatusCode::INTERNAL_SERVER_ERROR,
    //         AppAPIResponse::message(format!("unable to update User: {e}")),
    //     )),
    // }
}
//endregion

#[debug_handler]
#[utoipa::path(
delete,
path = "/{:id}",
summary = "delete a user record",
params(
("id" = i32, Path, description = "id for the user to delete")
),
responses(
(status = NO_CONTENT, description = "successful request"),
(status = BAD_REQUEST, description = "bad request", body = AppAPIResponse)),
tag = USERS_TAG
)]
pub async fn delete_user(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, AppAPIResponse)> {
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
        .is_authorized::<User>(
            &AuthorizableType::User(User {
                id: id,
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
        }
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppAPIResponse::message("Could not check user authorizations"),
        ));
    }
    //TODO: Implement database for auth
    return Err((
        StatusCode::NOT_IMPLEMENTED,
        AppAPIResponse::message("auth not implemented"),
    ));

    // let rs = sqlx::query_as!(
    //     User,
    //     "SELECT * FROM users where id = $1 and team_id = $2",
    //     id,
    //     user.team_id
    // )
    //     .fetch_one(&state.pool.clone())
    //     .await;
    // if let Ok(data) =rs {
    //     if let Err(e) = sqlx::query!("DELETE FROM users where id = $1", id)
    //         .execute(&state.pool)
    //         .await
    //     {
    //         error!("Unable to delete {0}: {1}", id, e);
    //         Err((
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             AppAPIResponse::message(format!("unable to delete user: {e}")),
    //         ))
    //     } else {
    //         Ok(StatusCode::NO_CONTENT)
    //     }
    // } else {
    //     Err((
    //         StatusCode::BAD_REQUEST,
    //         AppAPIResponse::message("user not found"),
    //     ))
    // }
}
