use crate::consts::*;
use crate::modules::users::User;
use crate::modules::{AppAPIResponse, Authorizable, AuthorizableType, RBAC};
use crate::state::AppState;
use crate::MODEL_HASH;
use async_trait::async_trait;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{debug_handler, Json};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::collections::HashMap;
use std::error::Error;
use sunspec_rs::json::group::Group;
use sunspec_rs::json::misc::JSONModel;
use sunspec_rs::sunspec_models::ModelSource;
use tower_sessions::Session;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub(crate) fn point_routes(state: AppState) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_point))
        .routes(routes!(get_all_points))
        .with_state(state)
}
pub struct Point;
/// a given point inside of a model
#[derive(Serialize, Deserialize, ToSchema, Debug, Default, Clone)]
pub struct PointResponse {
    /// model number
    pub model: i32,
    /// point name
    pub name: String,
    /// description of point, if available
    pub description: String,
}

/// A model containing all points associated with this model
#[derive(Serialize, Deserialize, ToSchema, Debug, Default, Clone)]
pub struct ModelResponse {
    /// model id
    pub model: i32,
    /// proper name for model
    pub name: String,
    /// description of model, if available
    pub description: String,
    /// list of points in this model
    pub points: Vec<PointResponse>,
}
/// A single unit containing a unit string and its array of models
#[derive(Serialize, Deserialize, ToSchema, Debug, Default, Clone)]
pub struct UnitResponse {
    /// unit in ip:port/slave_id format
    pub unit: String,
    /// list of models in this unit
    pub models: Vec<ModelResponse>,
}
/// A list of models and points for a specific unit
#[derive(Serialize, Deserialize, ToSchema, Debug, Default, Clone)]
pub struct UnitList {
    /// A list of unit-models
    units: Vec<UnitResponse>,
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
path = "/",
summary = "retrieve all available points",
responses(
(status = OK, description = "successful request", body = UnitList),
(status = BAD_REQUEST, description = "bad request", body = AppAPIResponse)),
tag = POINTS_TAG
)]
pub async fn get_all_points(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<UnitList>, (StatusCode, AppAPIResponse)> {
    Ok(get_all_points_from_hash().await)
}

#[cached(time = 10)]
async fn get_all_points_from_hash() -> Json<UnitList> {
    let mh = MODEL_HASH.read().await;
    let mut response_list: UnitList = UnitList::default();
    for (unit, models) in mh.iter() {
        let mut built_unit = UnitResponse::default();
        built_unit.unit = unit.clone();

        for (model, data) in models.iter() {
            let mut built_model = ModelResponse::default();
            built_model.model = *model as i32;
            if let ModelSource::Json(json) = data.clone().model.source {
                let g = json.group.clone();
                built_model.name = json.group.name.clone();
                built_model.description = json.group.desc.unwrap_or_default();
                let mut catalog = HashMap::new();
                iter_group(g, &mut catalog, None);
                for (k, v) in catalog.iter() {
                    built_model.points.push(PointResponse {
                        model: *model as i32,
                        name: k.clone(),
                        description: v.description.clone(),
                    })
                }
                built_unit.models.push(built_model);
            } else {
                built_model.name = data.model.model.name.clone();
                for block in data.model.model.block.iter() {
                    for point in block.point.iter() {
                        let mut desc = "".to_string();
                        if let Some(literal) = point.clone().literal {
                            desc = literal.description.unwrap_or_default();
                        }
                        built_model.points.push(PointResponse {
                            model: *model as i32,
                            name: point.clone().id,
                            description: desc,
                        })
                    }
                }
                built_unit.models.push(built_model);
            }
        }
        response_list.units.push(built_unit);
    }
    Json(response_list)
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

pub fn iter_group(
    group: Group,
    mut catalog: &mut HashMap<String, ModelResponse>,
    prefix: Option<String>,
) {
    let mut pointprefix = "".to_string();
    match prefix {
        Some(p) => pointprefix = format!("{}.{}", p, group.name.clone()),
        None => pointprefix = format!(".{}", group.name.clone()),
    };

    for point in group.points {
        let point_name = format!("{pointprefix}.{}", point.name.clone());
        catalog.insert(
            point_name,
            ModelResponse {
                model: 0,
                name: point.name.clone(),
                description: point.desc.unwrap_or_default(),
                points: vec![],
            },
        );
    }

    for g in group.groups {
        iter_group(g, &mut catalog, Some(pointprefix.clone()));
    }
}
