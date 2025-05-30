use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub(crate) struct ApiKey {
    pub(crate) id: i32,
    pub(crate) user_id: i32,
    pub(crate) key: String,
}
