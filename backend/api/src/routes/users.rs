use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{auth::middleware::AuthenticatedUser, auth_service::AuthService, AppState};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub encrypted_password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub encrypted_password: String,
    pub two_factor_code: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub two_factor_enabled: bool,
    pub force_password_change: bool,
}

#[derive(Serialize)]
pub struct PublicKeyResponse {
    pub public_key: String,
}

#[derive(Serialize)]
pub struct UserProfileResponse {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub two_factor_enabled: bool,
    pub force_password_change: bool,
}

#[derive(Serialize)]
pub struct Setup2FAResponse {
    pub secret: String,
    pub qr_code: String,
}

#[derive(Deserialize)]
pub struct TwoFactorCodeRequest {
    pub code: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ChangeEmailRequest {
    pub new_email: String,
}

pub fn users_routes() -> Router<AppState> {
    let auth_routes = Router::new()
        .route("/profile", get(get_user_profile))
        .route("/validate-session", get(validate_session))
        .route("/logout", post(logout_user))
        .route("/2fa/setup", post(setup_2fa))
        .route("/2fa/enable", post(enable_2fa))
        .route("/2fa/disable", post(disable_2fa))
        .route("/change-password", post(change_password))
        .route("/change-email", post(change_email));

    Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/public-key", get(get_public_key))
        .merge(auth_routes)
}

pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_service = AuthService::new(state.db_conn.clone());

    match auth_service
        .register_user(payload.username.clone(), payload.encrypted_password)
        .await
    {
        Ok(token) => match crate::auth::jwt::verify_jwt(&token) {
            Ok(token_data) => {
                let cookie = Cookie::build(("auth_token", token.clone()))
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .build();

                Ok((
                    StatusCode::OK,
                    [(header::SET_COOKIE, cookie.to_string())],
                    Json(AuthResponse {
                        token,
                        user_id: token_data.claims.user_id.to_string(),
                        username: token_data.claims.username,
                        two_factor_enabled: false,
                        force_password_change: false,
                    }),
                ))
            }
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
        Err(err) => {
            tracing::error!("Registration failed: {}", err);
            match err {
                crate::auth_service::AuthError::UserAlreadyExists => Err(StatusCode::CONFLICT),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_service = AuthService::new(state.db_conn.clone());

    match auth_service
        .login_user(
            payload.username,
            payload.encrypted_password,
            payload.two_factor_code,
        )
        .await
    {
        Ok((token, two_factor_enabled, force_password_change)) => {
            match crate::auth::jwt::verify_jwt(&token) {
                Ok(token_data) => {
                    let cookie = Cookie::build(("auth_token", token.clone()))
                        .path("/")
                        .http_only(true)
                        .same_site(SameSite::Strict)
                        .build();

                    Ok((
                        StatusCode::OK,
                        [(header::SET_COOKIE, cookie.to_string())],
                        Json(AuthResponse {
                            token,
                            user_id: token_data.claims.user_id.to_string(),
                            username: token_data.claims.username,
                            two_factor_enabled,
                            force_password_change,
                        }),
                    ))
                }
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(err) => {
            tracing::error!("Login failed: {}", err);
            match err {
                crate::auth_service::AuthError::UserNotFound
                | crate::auth_service::AuthError::InvalidCredentials => {
                    Err(StatusCode::UNAUTHORIZED)
                }
                crate::auth_service::AuthError::TwoFactorRequired => Err(StatusCode::FORBIDDEN),
                crate::auth_service::AuthError::InvalidTwoFactorCode => {
                    Err(StatusCode::UNAUTHORIZED)
                }
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

pub async fn logout_user(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
    axum::http::request::Parts { headers, .. }: axum::http::request::Parts,
) -> Result<Json<Value>, StatusCode> {
    let auth_service = AuthService::new(state.db_conn.clone());
    let exp_datetime = DateTime::from_timestamp(claims.exp, 0).unwrap_or_else(Utc::now);

    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .unwrap_or("")
        .to_string();

    match auth_service.logout_user(token, exp_datetime).await {
        Ok(_) => Ok(Json(json!({ "message": "Logged out successfully" }))),
        Err(err) => {
            tracing::error!("Logout failed: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_public_key(
    State(state): State<AppState>,
) -> Result<Json<PublicKeyResponse>, StatusCode> {
    let auth_service = AuthService::new(state.db_conn.clone());

    match auth_service.get_public_key("main").await {
        Ok(public_key) => Ok(Json(PublicKeyResponse { public_key })),
        Err(err) => {
            tracing::error!("Failed to get public key: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_user_profile(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<UserProfileResponse>, StatusCode> {
    let auth_service = AuthService::new(state.db_conn.clone());

    match auth_service.get_user_by_id(claims.user_id).await {
        Ok(user) => Ok(Json(UserProfileResponse {
            id: user.id.to_string(),
            username: user.name,
            email: user.email,
            two_factor_enabled: user.two_factor_enabled,
            force_password_change: user.force_password_change,
        })),
        Err(err) => {
            tracing::error!("Failed to get user profile: {}", err);
            match err {
                crate::auth_service::AuthError::UserNotFound => Err(StatusCode::NOT_FOUND),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

pub async fn validate_session(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<UserProfileResponse>, StatusCode> {
    let auth_service = AuthService::new(state.db_conn.clone());

    match auth_service.get_user_by_id(claims.user_id).await {
        Ok(user) => Ok(Json(UserProfileResponse {
            id: user.id.to_string(),
            username: user.name,
            email: user.email,
            two_factor_enabled: user.two_factor_enabled,
            force_password_change: user.force_password_change,
        })),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn setup_2fa(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Setup2FAResponse>, StatusCode> {
    let auth_service = AuthService::new(state.db_conn.clone());

    match auth_service.generate_2fa_secret(claims.user_id).await {
        Ok((secret, qr_code)) => Ok(Json(Setup2FAResponse { secret, qr_code })),
        Err(err) => {
            tracing::error!("Failed to setup 2FA: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn enable_2fa(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<TwoFactorCodeRequest>,
) -> Result<Json<Value>, StatusCode> {
    let auth_service = AuthService::new(state.db_conn.clone());

    match auth_service.enable_2fa(claims.user_id, payload.code).await {
        Ok(_) => Ok(Json(json!({ "message": "2FA enabled successfully" }))),
        Err(crate::auth_service::AuthError::InvalidTwoFactorCode) => Err(StatusCode::BAD_REQUEST),
        Err(err) => {
            tracing::error!("Failed to enable 2FA: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn disable_2fa(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<TwoFactorCodeRequest>,
) -> Result<Json<Value>, StatusCode> {
    let auth_service = AuthService::new(state.db_conn.clone());

    match auth_service.disable_2fa(claims.user_id, payload.code).await {
        Ok(_) => Ok(Json(json!({ "message": "2FA disabled successfully" }))),
        Err(crate::auth_service::AuthError::InvalidTwoFactorCode) => Err(StatusCode::BAD_REQUEST),
        Err(err) => {
            tracing::error!("Failed to disable 2FA: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn change_password(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, StatusCode> {
    let auth_service = AuthService::new(state.db_conn.clone());

    match auth_service
        .change_password(claims.user_id, payload.old_password, payload.new_password)
        .await
    {
        Ok(_) => Ok(Json(json!({ "message": "Password changed successfully" }))),
        Err(crate::auth_service::AuthError::InvalidCredentials) => Err(StatusCode::BAD_REQUEST),
        Err(err) => {
            tracing::error!("Failed to change password: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn change_email(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ChangeEmailRequest>,
) -> Result<Json<Value>, StatusCode> {
    let auth_service = AuthService::new(state.db_conn.clone());

    match auth_service
        .change_email(claims.user_id, payload.new_email)
        .await
    {
        Ok(_) => Ok(Json(json!({ "message": "Email changed successfully" }))),
        Err(crate::auth_service::AuthError::UserAlreadyExists) => Err(StatusCode::CONFLICT),
        Err(err) => {
            tracing::error!("Failed to change email: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
