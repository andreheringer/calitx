use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ConfigError {
    details: String,
}

impl ConfigError {
    pub fn new(msg: &str) -> ConfigError {
        ConfigError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Debug)]
pub struct CompressionError {
    details: String,
}

impl CompressionError {
    pub fn new(msg: &str) -> CompressionError {
        CompressionError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for CompressionError {
    fn description(&self) -> &str {
        &self.details
    }
}
