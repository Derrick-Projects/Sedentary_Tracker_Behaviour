// Database Integration Tests
// These tests verify that the persistence layer works correctly
// against the real PostgreSQL database.

use sqlx::postgres::PgPoolOptions;

// Test that we can connect to the database, insert a row, and read it back.
#[tokio::test]
async fn test_database_persistence() {
    // 1. Setup: Connect to the running Database
    let connection_string = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set!")
    });

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    // 2. Execution: Insert a row using SQLx
    let state = "ACTIVE";
    let timer_seconds = 0i32;
    let acceleration_val = 1.5f32;

    let inserted = sqlx::query!(
        r#"
        INSERT INTO sedentary_log (state, timer_seconds, acceleration_val)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        state,
        timer_seconds,
        acceleration_val
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert data");

    let row_id = inserted.id;

    // 3. Read it back to verify
    let saved = sqlx::query!(
        r#"SELECT state, timer_seconds, acceleration_val FROM sedentary_log WHERE id = $1"#,
        row_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch data");

    assert_eq!(saved.state, "ACTIVE");
    assert_eq!(saved.timer_seconds, Some(0));
    assert!((saved.acceleration_val.unwrap() - 1.5).abs() < 0.001);

    // 4. Cleanup
    sqlx::query!("DELETE FROM sedentary_log WHERE id = $1", row_id)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test data");
}

// Test inserting sedentary state data
#[tokio::test]
async fn test_sedentary_state_persistence() {
    let connection_string = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set!")
    });

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    let state = "STILL";
    let timer_seconds = 1800i32;
    let acceleration_val = 0.01f32;

    let inserted = sqlx::query!(
        r#"
        INSERT INTO sedentary_log (state, timer_seconds, acceleration_val)
        VALUES ($1, $2, $3)
        RETURNING id, created_at
        "#,
        state,
        timer_seconds,
        acceleration_val
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert sedentary data");

    // Verify the data was saved correctly
    assert!(
        inserted.created_at.is_some(),
        "Timestamp should be auto-generated"
    );

    // Cleanup
    sqlx::query!("DELETE FROM sedentary_log WHERE id = $1", inserted.id)
        .execute(&pool)
        .await
        .expect("Failed to cleanup");
}

// Test inserting fidget state data
#[tokio::test]
async fn test_fidget_state_persistence() {
    let connection_string = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set!")
    });

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    // Insert a FIDGET state
    let state = "FIDGET";
    let timer_seconds = 60i32;
    let acceleration_val = 0.3f32;

    let inserted = sqlx::query!(
        r#"
        INSERT INTO sedentary_log (state, timer_seconds, acceleration_val)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        state,
        timer_seconds,
        acceleration_val
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert fidget data");

    // verify
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM sedentary_log WHERE id = $1 AND state = 'FIDGET'"#,
        inserted.id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count");

    assert_eq!(count, Some(1));

    // Cleanup
    sqlx::query!("DELETE FROM sedentary_log WHERE id = $1", inserted.id)
        .execute(&pool)
        .await
        .expect("Failed to cleanup");
}

// Test querying the latest sedentary log entry
#[tokio::test]
async fn test_query_latest_entry() {
    let connection_string = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set!")
    });

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    // Insert two entries
    let id1 = sqlx::query_scalar!(
        r#"INSERT INTO sedentary_log (state, timer_seconds, acceleration_val) VALUES ('ACTIVE', 0, 2.0) RETURNING id"#
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert first");

    // Small delay to ensure different timestamps
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let id2 = sqlx::query_scalar!(
        r#"INSERT INTO sedentary_log (state, timer_seconds, acceleration_val) VALUES ('STILL', 100, 0.05) RETURNING id"#
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert second");

    // Query latest entry
    let latest =
        sqlx::query!(r#"SELECT id, state FROM sedentary_log ORDER BY created_at DESC LIMIT 1"#)
            .fetch_one(&pool)
            .await
            .expect("Failed to query latest");

    assert_eq!(latest.id, id2, "Should return the most recent entry");
    assert_eq!(latest.state, "STILL");

    // Cleanup
    sqlx::query!("DELETE FROM sedentary_log WHERE id IN ($1, $2)", id1, id2)
        .execute(&pool)
        .await
        .expect("Failed to cleanup");
}
