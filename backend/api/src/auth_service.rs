use chrono::{DateTime, Utc};
use entity::{invalid_jwt, key, user, InvalidJwt, Key, User};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use thiserror::Error;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

use crate::auth::{
    crypto::{
        decrypt_password, generate_salt, hash_password, verify_password, CryptoError, RsaKeyPair,
    },
    jwt::create_jwt,
};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("Crypto error: {0}")]
    CryptoError(#[from] CryptoError),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("2FA required")]
    TwoFactorRequired,
    #[error("Invalid 2FA code")]
    InvalidTwoFactorCode,
    #[error("Password change required")]
    #[allow(dead_code)]
    PasswordChangeRequired,
}

pub type AuthResult<T> = Result<T, AuthError>;

#[derive(Clone)]
pub struct AuthService {
    db: DatabaseConnection,
}

impl AuthService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn register_user(
        &self,
        username: String,
        encrypted_password: String,
    ) -> AuthResult<String> {
        let existing_user = User::find()
            .filter(user::Column::Name.eq(&username))
            .one(&self.db)
            .await?;

        if existing_user.is_some() {
            return Err(AuthError::UserAlreadyExists);
        }

        let rsa_key = self.get_or_create_rsa_key("main").await?;
        let password = decrypt_password(&encrypted_password, &rsa_key.private_key)?;
        let salt = generate_salt();
        let hashed_password = hash_password(&password, &salt)?;

        let user_id = Uuid::new_v4();
        let new_user = user::ActiveModel {
            id: ActiveValue::Set(user_id),
            name: ActiveValue::Set(username.clone()),
            password: ActiveValue::Set(hashed_password),
            salt: ActiveValue::Set(salt),
            email: ActiveValue::NotSet,
            two_factor_secret: ActiveValue::NotSet,
            two_factor_enabled: ActiveValue::Set(false),
            force_password_change: ActiveValue::Set(false),
        };

        User::insert(new_user)
            .exec_without_returning(&self.db)
            .await?;

        let token = create_jwt(user_id, username)?;
        Ok(token)
    }

    pub async fn login_user(
        &self,
        username: String,
        encrypted_password: String,
        two_factor_code: Option<String>,
    ) -> AuthResult<(String, bool, bool)> {
        let user = User::find()
            .filter(user::Column::Name.eq(&username))
            .one(&self.db)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        let rsa_key = self.get_or_create_rsa_key("main").await?;
        let password = decrypt_password(&encrypted_password, &rsa_key.private_key)?;

        if !verify_password(&password, &user.salt, &user.password)? {
            return Err(AuthError::InvalidCredentials);
        }

        if user.two_factor_enabled {
            if let Some(code) = two_factor_code {
                if !self.verify_2fa_code(&user, &code)? {
                    return Err(AuthError::InvalidTwoFactorCode);
                }
            } else {
                return Err(AuthError::TwoFactorRequired);
            }
        }

        let token = create_jwt(user.id, user.name)?;
        Ok((token, user.two_factor_enabled, user.force_password_change))
    }

    pub async fn logout_user(&self, token: String, exp: DateTime<Utc>) -> AuthResult<()> {
        let jwt_id = Uuid::new_v4();
        let invalid_jwt = invalid_jwt::ActiveModel {
            id: ActiveValue::Set(jwt_id),
            token: ActiveValue::Set(token),
            exp: ActiveValue::Set(exp.naive_utc()),
        };

        InvalidJwt::insert(invalid_jwt)
            .exec_without_returning(&self.db)
            .await?;
        Ok(())
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> AuthResult<user::Model> {
        User::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or(AuthError::UserNotFound)
    }

    pub async fn get_or_create_rsa_key(&self, key_name: &str) -> AuthResult<RsaKeyPair> {
        if let Some(existing_key) = Key::find()
            .filter(key::Column::Name.eq(key_name))
            .one(&self.db)
            .await?
        {
            return Ok(RsaKeyPair::from_private_key(&existing_key.private_key)?);
        }

        let key_pair = RsaKeyPair::generate()?;
        let key_id = Uuid::new_v4();
        let new_key = key::ActiveModel {
            id: ActiveValue::Set(key_id),
            name: ActiveValue::Set(key_name.to_string()),
            private_key: ActiveValue::Set(key_pair.private_key.clone()),
        };

        Key::insert(new_key)
            .exec_without_returning(&self.db)
            .await?;

        Ok(key_pair)
    }

    pub async fn get_public_key(&self, key_name: &str) -> AuthResult<String> {
        let key_pair = self.get_or_create_rsa_key(key_name).await?;
        Ok(key_pair.public_key)
    }

    pub async fn cleanup_expired_tokens(&self) -> AuthResult<()> {
        let now = Utc::now().naive_utc();
        InvalidJwt::delete_many()
            .filter(invalid_jwt::Column::Exp.lt(now))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn generate_2fa_secret(&self, user_id: Uuid) -> AuthResult<(String, String)> {
        let user = self.get_user_by_id(user_id).await?;

        let secret_bytes: Vec<u8> = (0..20).map(|_| rand::random::<u8>()).collect();
        let secret = Secret::Raw(secret_bytes);

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes().unwrap(),
            Some("App".to_string()),
            user.name.clone(),
        )
        .unwrap();

        let secret_string = secret.to_encoded().to_string();
        let qr_code_url = totp.get_qr_base64().unwrap();

        let mut user_active: user::ActiveModel = user.into();
        user_active.two_factor_secret = Set(Some(secret_string.clone()));
        user_active.update(&self.db).await?;

        Ok((secret_string, qr_code_url))
    }

    pub async fn enable_2fa(&self, user_id: Uuid, code: String) -> AuthResult<()> {
        let user = self.get_user_by_id(user_id).await?;

        if user.two_factor_secret.is_none() {
            return Err(AuthError::InvalidTwoFactorCode);
        }

        if !self.verify_2fa_code(&user, &code)? {
            return Err(AuthError::InvalidTwoFactorCode);
        }

        let mut user_active: user::ActiveModel = user.into();
        user_active.two_factor_enabled = Set(true);
        user_active.update(&self.db).await?;

        Ok(())
    }

    pub async fn disable_2fa(&self, user_id: Uuid, code: String) -> AuthResult<()> {
        let user = self.get_user_by_id(user_id).await?;

        if !user.two_factor_enabled {
            return Ok(());
        }

        if !self.verify_2fa_code(&user, &code)? {
            return Err(AuthError::InvalidTwoFactorCode);
        }

        let mut user_active: user::ActiveModel = user.into();
        user_active.two_factor_enabled = Set(false);
        user_active.two_factor_secret = Set(None);
        user_active.update(&self.db).await?;

        Ok(())
    }

    pub fn verify_2fa_code(&self, user: &user::Model, code: &str) -> AuthResult<bool> {
        if let Some(secret) = &user.two_factor_secret {
            let totp = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                Secret::Encoded(secret.clone()).to_bytes().unwrap(),
                Some("App".to_string()),
                user.name.clone(),
            )
            .unwrap();

            Ok(totp.check_current(code).unwrap_or(false))
        } else {
            Ok(false)
        }
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        old_encrypted_password: String,
        new_encrypted_password: String,
    ) -> AuthResult<()> {
        let user = self.get_user_by_id(user_id).await?;
        let rsa_key = self.get_or_create_rsa_key("main").await?;

        if !user.force_password_change && !old_encrypted_password.is_empty() {
            let old_password = decrypt_password(&old_encrypted_password, &rsa_key.private_key)?;
            if !verify_password(&old_password, &user.salt, &user.password)? {
                return Err(AuthError::InvalidCredentials);
            }
        }

        let new_salt = generate_salt();
        let new_password = decrypt_password(&new_encrypted_password, &rsa_key.private_key)?;
        let hashed_password = hash_password(&new_password, &new_salt)?;

        let mut user_active: user::ActiveModel = user.into();
        user_active.password = Set(hashed_password);
        user_active.salt = Set(new_salt);
        user_active.force_password_change = Set(false);
        user_active.update(&self.db).await?;

        Ok(())
    }

    pub async fn change_email(&self, user_id: Uuid, new_email: String) -> AuthResult<()> {
        let user = self.get_user_by_id(user_id).await?;

        if User::find()
            .filter(user::Column::Email.eq(&new_email))
            .one(&self.db)
            .await?
            .is_some()
        {
            return Err(AuthError::UserAlreadyExists);
        }

        let mut user_active: user::ActiveModel = user.into();
        user_active.email = Set(Some(new_email));
        user_active.update(&self.db).await?;

        Ok(())
    }
}
