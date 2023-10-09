use std::error::Error;
use axum::headers::HeaderValue;
use axum::http::StatusCode;
use tracing::error;
use crate::auth::AuthError::{MissingSecret, ServerError, WrongSecret};
use crate::config::SETTINGS;

pub fn auth(secret: Option<&HeaderValue>) -> Result<bool, AuthError> {
    if let Some(value) = secret {
        let key = SETTINGS
            .get_string("apikey")
            .map_err(|err| ServerError(Box::new(err)))?;
        if value.to_str().map_err(|err| ServerError(Box::new(err)))? == key.as_str() {
            Ok(true)
        } else {
            Err(WrongSecret)
        }
    } else {
        Err(MissingSecret)
    }
}

pub enum AuthError {
    WrongSecret,
    MissingSecret,
    ServerError(Box<dyn Error>),
}

impl AuthError {
    pub fn get(self) -> (StatusCode, &'static str) {
        match self {
            AuthError::WrongSecret => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingSecret => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::ServerError(err) => {
                error!("server error: {}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            },
        }
    }
}