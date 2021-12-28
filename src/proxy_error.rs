use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProxyError<'a> {
    message: &'a str,
}

impl<'a> ProxyError<'a> {
    pub fn new(message: &str) -> ProxyError {
        ProxyError { message }
    }
}

impl<'a> fmt::Display for ProxyError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Proxy error: {}", self.message)
    }
}

impl<'a> Error for ProxyError<'a> {
    fn description(&self) -> &str {
        "proxy error"
    }
}
