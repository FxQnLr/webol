use axum::http::{StatusCode, HeaderValue};
use axum::http::header::ToStrError;
use tracing::{debug, error, trace};
use crate::auth::Error::{MissingSecret, WrongSecret};
use crate::config::SETTINGS;

pub fn auth(secret: Option<&HeaderValue>) -> Result<bool, Error> {
    debug!("auth request with secret {:?}", secret);
    if let Some(value) = secret {
        trace!("value exists");
        let key = SETTINGS
            .get_string("apikey")
            .map_err(Error::Config)?;
        if value.to_str().map_err(Error::HeaderToStr)? == key.as_str() {
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

#[derive(Debug)]
pub enum Error {
    WrongSecret,
    MissingSecret,
    Config(config::ConfigError),
    HeaderToStr(ToStrError)
}

impl Error {
    pub fn get(self) -> (StatusCode, &'static str) {
        match self {
            Self::WrongSecret => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            Self::MissingSecret => (StatusCode::BAD_REQUEST, "Missing credentials"),
            Self::Config(err) => {
                error!("server error: {}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            },
            Self::HeaderToStr(err) => {
                error!("server error: {}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            },
        }
    }
}
