// src/bridge/acp/error.rs
//! ACP error types

use serde::{Deserialize, Serialize};

/// ACP protocol errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpError {
    pub code: AcpErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl AcpError {
    pub fn new(code: AcpErrorCode, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl std::fmt::Display for AcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for AcpError {}

/// ACP error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpErrorCode {
    MalformedMessage,
    UnknownReceiver,
    NotAuthorized,
    NotFound,
    InvalidPayload,
    Timeout,
    InternalError,
}

impl AcpErrorCode {
    pub fn to_code(self) -> u16 {
        match self {
            Self::MalformedMessage => 1001,
            Self::UnknownReceiver => 1002,
            Self::NotAuthorized => 1003,
            Self::NotFound => 1004,
            Self::InvalidPayload => 1005,
            Self::Timeout => 1006,
            Self::InternalError => 1999,
        }
    }
}
