use crate::consts::*;
use crate::modules::AppAPIResponse;
use crate::state::AppState;
use crate::API_DOC;
use axum::http::StatusCode;
use axum::Json;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub const USERS_TAG: &str = "users";
pub const USERS_TAG_DESCRIPTION: &str = "Operations about users";

pub const ROUTE_TAG: &str = "routes";
pub const ROUTE_TAG_DESCRIPTION: &str = "Operations about routes";

#[derive(OpenApi)]
#[openapi(
    tags(
    (name = USERS_TAG, description = USERS_TAG_DESCRIPTION ),
    )
)]
pub struct ApiDoc;

pub fn register_routes(state: AppState) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(health))
        .with_state(state)
}

#[utoipa::path(
    method(get, head),
    path = HEALTH_PATH,
    tag = ROUTE_TAG,
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
pub async fn health() -> &'static str {
    "ok"
}
/// Return JSON version of an OpenAPI schema
#[utoipa::path(
    get,
    path = API_PATH,
    responses(
        (status = 200, description = "JSON file", body = ())
    )
)]
pub async fn openapi() -> Result<Json<utoipa::openapi::OpenApi>, (StatusCode, AppAPIResponse)> {
    if let Some(api) = API_DOC.get() {
        Ok(Json(api.clone()))
    } else {
        error!("There was no api document stored in API_DOC.");
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppAPIResponse::message("API document not found"),
        ))
    }
}
