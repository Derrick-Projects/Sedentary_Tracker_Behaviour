use crate::models::ProcessedState;
use sqlx::PgPool;
use std::env;
use tokio::sync::broadcast;
use uuid::Uuid;

pub async fn spawn_db_worker(pool: PgPool, mut rx: broadcast::Receiver<String>) {
    tokio::spawn(async move {
        println!("Logic Logger Started...");

        while let Ok(json_msg) = rx.recv().await {
            // We deserialize the PROCESSED output, not the raw input
            if let Ok(data) = serde_json::from_str::<ProcessedState>(&json_msg) {
                // Save to 'sedentary_log'
                // We use valid data derived from our Logic Engine
                let result = sqlx::query!(
                    r#"
                    INSERT INTO sedentary_log (state, timer_seconds, acceleration_val)
                    VALUES ($1, $2, $3)
                    "#,
                    data.state,
                    data.timer as i32,
                    data.val
                )
                .execute(&pool)
                .await;

                if let Err(e) = result {
                    eprintln!("DB Error (sedentary_log): {}", e);
                }

                // Mirror to sensor_data for user-level statistics (if DEFAULT_USER_ID is set)
                if let Ok(default_user) = env::var("DEFAULT_USER_ID") {
                    if let Ok(user_uuid) = Uuid::parse_str(&default_user) {
                        let sensor_result = sqlx::query!(
                            r#"
                            INSERT INTO sensor_data (user_id, state, timer_seconds, acceleration_val, alert_triggered, timestamp)
                            VALUES ($1, $2, $3, $4, $5, $6)
                            "#,
                            user_uuid,
                            data.state,
                            data.timer as i32,
                            data.val,
                            data.alert,
                            data.timestamp
                        )
                        .execute(&pool)
                        .await;

                        if let Err(e) = sensor_result {
                            eprintln!("DB Error (sensor_data): {}", e);
                        }
                    }
                }
            }
        }
    });
}
