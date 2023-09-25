use crate::payload::{HAConfigPayload, StatePayload};
use anyhow::{bail, Result};
use lazy_static::lazy_static;
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::pool::PoolConnection;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{
    migrate::MigrateDatabase, ConnectOptions, Database, Encode, Error, Executor, FromRow, Pool,
    Row, Sqlite, SqlitePool,
};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::OnceCell;
use tracing_log::log::LevelFilter;
use url::Url;

#[derive(Debug, Clone, FromRow)]
pub struct point_history {
    uniqueid: String,
    value: String,
    timestamp: String,
}

const DB_URL: &str = "sqlite://sunspec_gateway.db";

lazy_static! {
    static ref DB_POOL: OnceCell<Pool<Sqlite>> = OnceCell::new();
}
pub async fn acquire_db() -> PoolConnection<Sqlite> {
    DB_POOL.get().unwrap().acquire().await.unwrap()
}

pub async fn create_db() -> Result<()> {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        info!("Creating database {}", DB_URL);
        return match Sqlite::create_database(DB_URL).await {
            Ok(_) => Ok(()),
            Err(error) => bail!("Create database error: {error}"),
        };
    };
    Ok(())
}

pub async fn prepare_to_database() -> anyhow::Result<()> {
    if let Err(e) = create_db().await {
        bail!(e);
    }
    let conn_options = SqliteConnectOptions::from_url(&Url::from_str(DB_URL).unwrap()).unwrap();
    //.log_statements(LevelFilter::Info);
    let pool = match SqlitePool::connect_with(conn_options).await {
        Ok(pool) => pool,
        Err(e) => {
            bail!(e);
        }
    };
    DB_POOL.set(pool).unwrap();
    info!("Executing migrations.");
    let mut conn = acquire_db().await;
    match sqlx::migrate!().run(&mut conn).await {
        Ok(_) => {
            info!("Migrations successful.");
        }
        Err(e) => {
            bail!(e);
        }
    }
    Ok(())
}

pub async fn cull_records_to(uniqueid: String, cull_num: u8) -> anyhow::Result<()> {
    let pool = DB_POOL.get().unwrap();
    match sqlx::query(
        r#"
        DELETE FROM point_history
        WHERE uniqueid = $1
        AND
        timestamp not in
        (SELECT timestamp from point_history
         WHERE
         uniqueid = $1
         ORDER BY timestamp LIMIT $2)
    "#,
    )
    .bind(uniqueid)
    .bind(cull_num)
    .execute(pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            bail!(e);
        }
    }
    //Ok(())
}

pub async fn check_needs_adjust(uniques_present: Vec<String>) -> anyhow::Result<Vec<String>> {
    let mut pool = DB_POOL.get().unwrap();
    let split_vec = match uniques_present.get(0).clone() {
        Some(s) => s,
        None => {
            return Ok(vec![]);
        }
    };
    let mut splitval = split_vec.splitn(4, ".");
    let (sn, mn, pn, st) = (
        splitval.next().unwrap().to_string(),
        splitval.next().unwrap().to_string(),
        splitval.next().unwrap().to_string(),
        splitval.next().unwrap().to_string(),
    );
    let uniqueid_prefix = format!("{sn}.{mn}.{pn}.%");
    let uniquebundle = uniques_present
        .iter()
        .map(|x| format!("'{x}'"))
        .collect::<Vec<String>>()
        .join(",");
    let mut response: Vec<String> = vec![];
    let query_str: String = format!(
        r#"
    SELECT DISTINCT uniqueid,value from point_history
    WHERE uniqueid LIKE $1
    AND value = '"on"'
    AND uniqueid not in ({uniquebundle})
    ORDER BY TIMESTAMP DESC
    "#
    );

    let values = match sqlx::query(&query_str)
        .bind(uniqueid_prefix)
        .fetch_all(pool)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            bail!(e)
        }
    };
    for row in values {
        match row.try_get::<String, &str>("uniqueid") {
            Ok(s) => {
                if check_on(s.clone()).await {
                    response.push(s);
                }
            }
            Err(e) => {
                bail!(e);
            }
        };
    }
    Ok(response)
}

pub async fn check_on(uniqueid: String) -> bool {
    let pool = DB_POOL.get().unwrap();
    match sqlx::query(
        r#"
    SELECT uniqueid, value from point_history
    WHERE uniqueid = $1
    ORDER BY timestamp
    LIMIT 1
    "#,
    )
    .bind(uniqueid)
    .fetch_one(pool)
    .await
    {
        Ok(v) => {
            let val = v.get::<String, &str>("value");
            val == "on"
        }
        Err(e) => false,
    }
}

pub async fn write_payload_history(
    config: HAConfigPayload,
    state: StatePayload,
) -> anyhow::Result<()> {
    let timestamp = state.last_seen;
    let uniqueid = config.unique_id;
    let value_json = match serde_json::to_string(&state.value) {
        Ok(s) => s,
        Err(e) => {
            bail!(e);
        }
    };
    let mut pool = DB_POOL.get().unwrap();
    match sqlx::query(
        r#"
    INSERT INTO point_history (timestamp, uniqueid, value)
    VALUES (?,?,?)
    "#,
    )
    .bind(timestamp.timestamp())
    .bind(uniqueid)
    .bind(value_json)
    .execute(pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            bail!(e);
        }
    }
}
