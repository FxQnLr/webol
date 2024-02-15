use axum::http::{StatusCode, HeaderValue};
use axum::http::header::ToStrError;
use tracing::{debug, error, trace};
use crate::auth::Error::{MissingSecret, WrongSecret};
use crate::config::Config;

pub fn auth(config: &Config, secret: Option<&HeaderValue>) -> Result<bool, Error> {
    debug!("auth request with secret {:?}", secret);
    if let Some(value) = secret {
        trace!("value exists");
        let key = &config.apikey;
        if value.to_str()? == key.as_str() {
            debug!("successful auth");
            Ok(true)
        } else {
            debug!("unsuccessful auth (wrong secret)");
            Err(WrongSecret)
        }
    } else {
        debug!("unsuccessful auth (no secret)");
        Err(MissingSecret)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("wrong secret")]
    WrongSecret,
    #[error("missing secret")]
    MissingSecret,
    #[error("parse error: {source}")]
    HeaderToStr {
        #[from]
        source: ToStrError
    }
}

impl Error {
    pub fn get(self) -> (StatusCode, &'static str) {
        match self {
            Self::WrongSecret => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            Self::MissingSecret => (StatusCode::BAD_REQUEST, "Missing credentials"),
            Self::HeaderToStr { source } => {
                error!("auth: {}", source);
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            },
        }
    }
}
