use crate::{auth::create_jwt, state::AppState};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use redis::AsyncCommands;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

pub async fn show_login_form() -> Redirect {
    Redirect::permanent("/login.html")
}

pub async fn login_handler(State(state): State<AppState>, Form(form): Form<LoginForm>) -> Response {
    // Rate limiting: check failed login attempts per email
    let rate_limit_key = format!("login_attempts:{}", form.email);
    let max_attempts = 5;
    let attempt_window = 60; 

    let mut redis_conn = match state.redis.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Service unavailable".to_string(),
            )
                .into_response();
        }
    };

    let attempts: i32 = redis_conn.get(&rate_limit_key).await.unwrap_or(0);
    if attempts >= max_attempts {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many failed login attempts. Please try again later.".to_string(),
        )
            .into_response();
    }

    // Dummy hash for timing attack mitigation
    let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$dummy$dummy";

    // Fetch user by email
    let user_result = sqlx::query!(
        r#"SELECT user_id, password_hash, name FROM users WHERE email = $1"#,
        form.email
    )
    .fetch_optional(&state.db)
    .await;

    let (user_exists, user_id, user_name, password_hash) = match user_result {
        Ok(Some(user)) => (
            true,
            Some(user.user_id.to_string()),
            Some(user.name),
            user.password_hash,
        ),
        Ok(None) => (false, None, None, dummy_hash.to_string()),
        Err(e) => {
            eprintln!("Database error: {e:?}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error.".to_string(),
            )
                .into_response();
        }
    };

    // Parse stored hash (or dummy hash if user doesn't exist)
    let parsed_hash = match PasswordHash::new(&password_hash) {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Corrupt password hash".to_string(),
            )
                .into_response();
        }
    };

    // Verify password (timing-safe: runs regardless of user existence)
    let valid = Argon2::default()
        .verify_password(form.password.as_bytes(), &parsed_hash)
        .is_ok();

    if user_exists && valid {
        // Clear rate limit counter on successful login
        let _: () = redis_conn.del(&rate_limit_key).await.unwrap_or(());

        match create_jwt(&user_id.unwrap(), &user_name.unwrap()) {
            Ok(token) => (StatusCode::OK, format!("{{\"token\":\"{}\"}}", token)).into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate token".to_string(),
            )
                .into_response(),
        }
    } else {
        // Increment failed attempt counter
        let _: () = redis_conn.incr(&rate_limit_key, 1).await.unwrap_or(());
        let _: () = redis_conn
            .expire(&rate_limit_key, attempt_window)
            .await
            .unwrap_or(());

        (
            StatusCode::UNAUTHORIZED,
            "Invalid email or password.".to_string(),
        )
            .into_response()
    }
}
