use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

// export sqlx errors so other crates can see them
pub use sqlx::Error;

// The main entry point for the database connection pool.

pub async fn get_db_pool(connection_string: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_string)
        .await
}

// The sqlx::FromRow trait allows us to fetch data directly into this struct.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Observation {
    pub id: Uuid,
    pub status: String,

    pub code_system: String,
    pub code_code: String,
    pub code_display: String,

    pub device_id: String,

    pub effective_date_time: DateTime<Utc>,

    pub value_value: Option<f64>,
    pub value_unit: Option<String>,
    pub value_system: Option<String>,
    pub value_code: Option<String>,

    pub components: Option<sqlx::types::Json<serde_json::Value>>,

    pub created_at: DateTime<Utc>,
}

// A "New" Observation struct for inserting data without ID or created_at
pub struct NewObservation {
    pub status: String,
    pub code_code: String,
    pub device_id: String,
    pub effective_date_time: DateTime<Utc>,
    pub value_value: Option<f64>,
    pub components: Option<serde_json::Value>,
}

impl NewObservation {
    pub async fn save(self, pool: &Pool<Postgres>) -> Result<Uuid, sqlx::Error> {
        let rec = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO observations (
                status, 
                code_system, code_code, code_display, 
                device_id, effective_date_time, 
                value_value, value_unit, value_system, value_code, 
                components
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::DOUBLE PRECISION, $8, $9, $10, $11)
            RETURNING id
            "#,
        )
        .bind(&self.status)
        .bind("http://loinc.org")
        .bind(&self.code_code)
        .bind("Sedentary behavior duration")
        .bind(&self.device_id)
        .bind(self.effective_date_time)
        .bind(self.value_value)
        .bind("s")
        .bind("http://unitsofmeasure.org")
        .bind("s")
        .bind(&self.components)
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }
}
