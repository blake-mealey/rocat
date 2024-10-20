use std::future::Future;

use log::debug;
use parking_lot::RwLock;
use reqwest::{
    cookie::{CookieStore, Jar},
    header::{HeaderMap, HeaderValue},
    RequestBuilder, Response, StatusCode,
};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum RobloxAuthError {
    #[error("Unable to find ROBLOSECURITY cookie. Login to Roblox Studio or set the ROBLOSECURITY environment variable.")]
    MissingRoblosecurityCookie,
}

// Temporary to make the new errors backwards compatible with the String errors throughout the project.
impl From<RobloxAuthError> for String {
    fn from(e: RobloxAuthError) -> Self {
        e.to_string()
    }
}

pub struct RobloxCookieStore(Jar);

impl RobloxCookieStore {
    pub fn new() -> Result<Self, RobloxAuthError> {
        let roblosecurity_cookie =
            rbx_cookie::get().ok_or(RobloxAuthError::MissingRoblosecurityCookie)?;

        let jar = Jar::default();
        let url = "https://roblox.com".parse::<Url>().unwrap();
        jar.add_cookie_str(&roblosecurity_cookie, &url);

        Ok(Self(jar))
    }
}

impl CookieStore for RobloxCookieStore {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        self.0.set_cookies(cookie_headers, url)
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        self.0.cookies(url)
    }
}

#[derive(Error, Debug)]
pub enum CsrfTokenRequestError {
    #[error("Failed to create request: {0}")]
    RequestFactoryError(#[from] anyhow::Error),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
}

const CSRF_TOKEN_HEADER_NAME: &str = "X-CSRF-TOKEN";

pub struct RobloxCsrfTokenStore(RwLock<Option<HeaderValue>>);

impl RobloxCsrfTokenStore {
    pub fn new() -> Self {
        Self(RwLock::new(None))
    }

    /**
     * Updates the auth instance's CSRF token from a response's headers map if it is different from the
     * current value. Returns a boolean indicating whether a new value was found from the headers map.
     */
    pub fn set_csrf_token_from_headers(&self, headers: &HeaderMap) -> bool {
        let headers_csrf_token = headers.get(CSRF_TOKEN_HEADER_NAME);
        let new_value = match (self.0.read().as_ref(), headers_csrf_token) {
            (None, Some(new_value)) => Some(new_value.clone()),
            (Some(prev_value), Some(new_value)) if prev_value != new_value => {
                Some(new_value.clone())
            }
            _ => None,
        };

        match new_value {
            Some(value) => {
                debug!("Store CSRF token: {}", value.to_str().unwrap_or("INVALID"));
                self.0.write().replace(value);
                true
            }
            None => false,
        }
    }

    pub async fn send_request<F, Fut>(
        &self,
        req_factory: F,
    ) -> Result<Response, CsrfTokenRequestError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = anyhow::Result<RequestBuilder>>,
    {
        let req = match self.0.read().as_ref() {
            Some(value) => req_factory().await?.header(CSRF_TOKEN_HEADER_NAME, value),
            None => req_factory().await?,
        };
        let res = req.send().await?;

        let has_new_token = self.set_csrf_token_from_headers(res.headers());

        match (res.status(), has_new_token) {
            // If the response was forbidden and we have a new CSRF token, retry once
            (StatusCode::FORBIDDEN, true) => match self.0.read().as_ref() {
                Some(value) => {
                    debug!(
                        "Retry Forbidden request with new CSRF token: {}",
                        value.to_str().unwrap_or("INVALID")
                    );
                    let req = req_factory().await?.header(CSRF_TOKEN_HEADER_NAME, value);
                    req.send().await.map_err(|e| e.into())
                }
                None => Ok(res),
            },
            _ => Ok(res),
        }
    }
}
