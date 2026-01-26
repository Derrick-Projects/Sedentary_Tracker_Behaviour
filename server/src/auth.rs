use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

fn jwt_secret() -> Vec<u8> {
    env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set!")
        .into_bytes()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub name: String,
    pub exp: usize,
}

pub fn create_jwt(user_id: &str, name: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 3600; // 1 hour

    let claims = Claims {
        sub: user_id.to_owned(),
        name: name.to_owned(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&jwt_secret()),
    )
}

#[derive(Debug)]
pub struct AuthUser {
    pub user_id: String,
    #[allow(dead_code)]
    pub name: String,
}

/// Custom rejection
pub struct AuthError {
    pub message: &'static str,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            [(
                "WWW-Authenticate",
                r#"Bearer realm="Sedentary Tracker", error="invalid_token""#,
            )],
            self.message,
        )
            .into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok());

        let header = match auth_header {
            Some(h) if h.starts_with("Bearer ") => h,
            _ => {
                return Err(AuthError {
                    message: "Missing Authorization header",
                })
            }
        };

        let token = &header[7..];
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(&jwt_secret()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| AuthError {
            message: "Invalid token",
        })?;

        Ok(AuthUser {
            user_id: token_data.claims.sub,
            name: token_data.claims.name.clone(),
        })
    }
}
