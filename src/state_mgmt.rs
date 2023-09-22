use sqlx::{migrate::MigrateDatabase, FromRow, Row, Sqlite, SqlitePool};

#[derive(Debug, Clone, FromRow)]
pub struct point_history {
    sn: String,
    model: String,
    point_name: String,
    value: String,
    timestamp: String,
}

const DB_URL: &str = "sqlite://sqlite.db";

pub async fn create_db() {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        info!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => info!("Create db success"),
            Err(error) => panic!("error: {error}"),
        }
    }
}
