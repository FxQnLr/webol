use axum::http::HeaderValue;
use tracing::{debug, trace};
use crate::config::Config;
use crate::error::Error;

pub fn auth(config: &Config, secret: Option<&HeaderValue>) -> Result<Response, Error> {
    debug!("auth request with secret {:?}", secret);
    let res = if let Some(value) = secret {
        trace!("auth value exists");
        let key = &config.apikey;
        if value.to_str()? == key.as_str() {
            debug!("successful auth");
            Response::Success
        } else {
            debug!("unsuccessful auth (wrong secret)");
            Response::WrongSecret
        }
    } else {
        debug!("unsuccessful auth (no secret)");
        Response::MissingSecret
    };
    Ok(res)
}

#[derive(Debug)]
pub enum Response {
    Success,
    WrongSecret,
    MissingSecret
}
