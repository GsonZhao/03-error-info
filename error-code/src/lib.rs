use std::{hash::{DefaultHasher, Hash, Hasher}, str::FromStr};

use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
pub use error_code_derive::ToErrorInfo;
pub struct ErrorInfo<T> {
    pub app_code: T,        // could be HTTP 400 bad request
    pub code: &'static str, // something link 01E739
    pub hash: String,
    pub client_msg: &'static str,
    pub server_msg: String,
}

pub trait ToErrorInfo {
    type T: FromStr;
    fn to_error_info(&self) -> ErrorInfo<Self::T>;
}

impl<T> ErrorInfo<T>
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    pub fn new(
        app_code: &str,
        code: &'static str,
        client_msg: &'static str,
        server_msg: impl std::fmt::Display,
    ) -> Self{
        let server_msg = server_msg.to_string();
        let mut hasher = DefaultHasher::new();
        server_msg.hash(&mut hasher);
        let hash = hasher.finish();
        let hash = BASE64_URL_SAFE_NO_PAD.encode(&hash.to_be_bytes());
        
        Self {
            app_code: T::from_str(app_code).expect("Invalid app code"),
            code,
            hash,
            client_msg,
            server_msg: server_msg.to_string(),
        }
    }
}

impl<T> ErrorInfo<T>
{
    pub fn client_msg(&self) -> &str {
        if self.client_msg.is_empty() {
            self.server_msg.as_str()
        } else {
            self.client_msg
        }       
    }
}


impl<T> std::fmt::Display for ErrorInfo<T>
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}-{}]: {}", self.code, self.hash, self.client_msg())
    }
}

impl<T> std::fmt::Debug for ErrorInfo<T>
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}-{}]: {}", self.code, self.hash, self.server_msg)
    }
}