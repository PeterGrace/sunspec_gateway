use sqlx::sqlite::SqlitePool;

use crate::auth::token_extractor::JwksCache;
use crate::modules::users::User;
use cached::UnboundCache;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub(crate) struct AppState {
    // we use the static ref DB_POOL for the sqlitepool object
    //pub(crate) pool: Option<SqlitePool>,
    pub(crate) jwks_cache: JwksCache,
    pub(crate) user_cache: Option<Arc<RwLock<UnboundCache<String, User>>>>,
}
