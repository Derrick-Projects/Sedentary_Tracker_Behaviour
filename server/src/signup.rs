use crate::state::AppState;
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

#[derive(Deserialize)]
pub struct SignUpForm {
    pub email: String,
    pub name: String,
    pub password: String,
}

pub async fn show_signup_form() -> Redirect {
    Redirect::permanent("/signup.html")
}

pub async fn signup_handler(
    State(state): State<AppState>,
    Form(form): Form<SignUpForm>,
) -> impl IntoResponse {
    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(form.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password"),
    };

    // Insert user
    let result = sqlx::query!(
        r#"
        INSERT INTO users (user_id, email, name, password_hash, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        uuid::Uuid::new_v4(),
        form.email,
        form.name,
        password_hash,
        chrono::Utc::now()
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (StatusCode::OK, "Welcome! You can now log in."),
        Err(e) => {
            eprintln!("Failed to insert user: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Could not sign up")
        }
    }
}
