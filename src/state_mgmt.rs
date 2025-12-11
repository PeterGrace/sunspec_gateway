use crate::payload::{HAConfigPayload, StatePayload};
use anyhow::{bail, Result};
use lazy_static::lazy_static;

use sqlx::pool::PoolConnection;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{migrate::MigrateDatabase, ConnectOptions, FromRow, Pool, Row, Sqlite, SqlitePool};
use std::str::FromStr;

use tokio::sync::OnceCell;

use url::Url;

#[derive(Debug, Clone, FromRow)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub struct point_history {
    uniqueid: String,
    value: String,
    timestamp: String,
}

#[derive(Default, Debug, Clone, FromRow)]
pub struct BitfieldHistory {
    pub uniqueid: String,
    pub field_name: String,
}

#[derive(Default, Debug, Clone, FromRow)]
pub struct AggregatedMeasurements {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub median: f64,
    pub count: i16,
    pub stdev: f64,
}

//const DB_URL: &str = "sqlite://sunspec_gateway.db";

lazy_static! {
    static ref DB_POOL: OnceCell<Pool<Sqlite>> = OnceCell::new();
    static ref DB_URL: OnceCell<String> = OnceCell::new();
}
pub async fn acquire_db() -> PoolConnection<Sqlite> {
    DB_POOL.get().unwrap().acquire().await.unwrap()
}

pub async fn create_db() -> Result<()> {
    let url = DB_URL.get().unwrap();
    if !Sqlite::database_exists(url).await.unwrap_or(false) {
        info!("Creating database {}", url);
        return match Sqlite::create_database(url).await {
            Ok(_) => Ok(()),
            Err(error) => bail!("Create database error: {error}"),
        };
    };
    Ok(())
}

pub async fn prepare_to_database() -> anyhow::Result<()> {
    let dbpath = match std::env::var("DB_FILE_PATH") {
        Ok(s) => s,
        Err(_e) => "sqlite://./sunspec_gateway.db".to_string(),
    };
    DB_URL.set(dbpath).unwrap();

    if let Err(e) = create_db().await {
        bail!(e);
    }
    let url = DB_URL.get().unwrap();
    let conn_options =
        SqliteConnectOptions::from_url(&Url::from_str(url).unwrap())?.extension("./stats");
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
    let pool = DB_POOL.get().unwrap();
    let split_vec = match uniques_present.get(0).clone() {
        Some(s) => s,
        None => {
            return Ok(vec![]);
        }
    };
    let mut splitval = split_vec.splitn(4, ".");
    let (sn, mn, pn, _st) = (
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
    ORDER BY timestamp desc
    LIMIT 1
    "#,
    )
    .bind(uniqueid)
    .fetch_one(pool)
    .await
    {
        Ok(v) => {
            let val = v.get::<String, &str>("value");
            val == r#""on""#
        }
        Err(_e) => false,
    }
}
pub async fn get_bitfield_history(uniqueid: &String) -> Vec<String> {
    if let Some(pool) = DB_POOL.get() {
        let values: Vec<String> = match sqlx::query_scalar(
            r#"
    SELECT field_name from bitfield_history
    WHERE uniqueid = $1
    "#,
        )
        .bind(uniqueid)
        .fetch_all(pool)
        .await
        {
            Ok(v) => v,
            Err(e) => {
                warn!("Error fetching bitfield history: {}", e);
                return vec![];
            }
        };
        values
    } else {
        vec![]
    }
}
pub async fn write_bitfield_history(uniqueid: &String, field_name: &String) -> anyhow::Result<()> {
    if let Some(pool) = DB_POOL.get() {
        match sqlx::query(
            r#"
    INSERT OR IGNORE INTO bitfield_history (uniqueid, field_name)
    VALUES ($1, $2)
    "#,
        )
        .bind(uniqueid)
        .bind(field_name)
        .execute(pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::Error::new(e)),
        }
    } else {
        Err(anyhow::anyhow!("Database pool not initialized"))
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
    let pool = DB_POOL.get().unwrap();
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

pub async fn get_history(uniqueid: String) -> anyhow::Result<AggregatedMeasurements> {
    let pool = DB_POOL.get().unwrap();

    // let mut ts: Vec<f64> = vec![];
    // let tsq= sqlx::query(
    //     r#"
    // SELECT CAST(value as real) as value
    // FROM point_history
    // WHERE uniqueid = $1
    // ORDER BY timestamp desc
    // "#
    // )
    //     .bind(uniqueid.clone())
    //     .fetch_all(pool)
    //     .await.unwrap();
    //
    // let _ = tsq.iter().map(|x| ts.push(x.get::<f64, &str>("value")));
    // let mut resl: Vec<f64> = vec![];
    // res_l(ts.into_iter(), resl.as_mut_slice());
    // info!("Reservoir Samples: {:#?}", resl);

    let values: AggregatedMeasurements = match sqlx::query_as(
        r#"
    SELECT
    COUNT(value) as count,
    MIN(CAST(value as real)) as min,
    MAX(CAST(value as real)) as max,
    PERCENTILE(CAST(value as real),50) as median,
    AVG(CAST(value as real)) as avg,
    STDDEV(CAST(value as real)) as stdev
    from point_history
    WHERE uniqueid = $1
    "#,
    )
    .bind(uniqueid)
    .fetch_one(pool)
    .await
    {
        Ok(s) => s,
        Err(e) => {
            bail!(e)
        }
    };

    Ok(values)
}
