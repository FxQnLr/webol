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
    HeaderToStr(ToStrError)
}

impl Error {
    pub fn get(self) -> (StatusCode, &'static str) {
        match self {
            Self::WrongSecret => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            Self::MissingSecret => (StatusCode::BAD_REQUEST, "Missing credentials"),
            Self::HeaderToStr(err) => {
                error!("server error: {}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            },
        }
    }
}
