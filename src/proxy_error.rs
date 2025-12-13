use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProxyError {
    message: String,
}

impl ProxyError {
    pub fn new(message: &str) -> ProxyError {
        ProxyError {
            message: message.to_string()
        }
    }
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Proxy error: {}", self.message)
    }
}

impl Error for ProxyError {
    fn description(&self) -> &str {
        "proxy error"
    }
}
