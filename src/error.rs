//! 翻译错误类型定义

use std::fmt;

/// 翻译错误类型
#[derive(Debug)]
pub enum TranslationError {
    /// 网络请求错误
    NetworkError(reqwest::Error),
    /// HTTP 状态错误
    HttpError {
        status: reqwest::StatusCode,
        body: String,
    },
    /// 认证错误
    AuthenticationError(String),
    /// 超时错误
    TimeoutError,
    /// 重试次数耗尽（包含每次尝试的错误信息）
    MaxRetriesExceeded {
        attempts: u32,
        errors: Vec<TranslationError>, // 记录每次重试的错误
    },
    /// 翻译服务返回的错误
    ServiceError(String),
    /// 配置错误
    ConfigurationError(String),
    /// 其他错误
    Other(String),
}

impl TranslationError {
    /// 判断错误是否可以重试
    pub fn is_retryable(&self) -> bool {
        match self {
            TranslationError::NetworkError(_) => true,
            TranslationError::HttpError { status, .. } => {
                // 5xx 状态码通常是服务器端问题，可以重试
                status.is_server_error()
            }
            TranslationError::TimeoutError => true,
            // 其他错误类型，如认证、配置、服务错误等，通常不可重试
            _ => false,
        }
    }
}

impl fmt::Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranslationError::NetworkError(e) => write!(f, "Network error: {}", e),
            TranslationError::HttpError { status, body } => {
                write!(f, "HTTP error {}: {}", status, body)
            }
            TranslationError::AuthenticationError(msg) => {
                write!(f, "Authentication error: {}", msg)
            }
            TranslationError::TimeoutError => write!(f, "Request timeout"),
            TranslationError::MaxRetriesExceeded { attempts, errors } => {
                writeln!(f, "Max retries exceeded after {} attempts", attempts)?;
                for (i, error) in errors.iter().enumerate() {
                    writeln!(f, "  Attempt {}: {}", i + 1, error)?;
                }
                Ok(())
            }
            TranslationError::ServiceError(msg) => write!(f, "Service error: {}", msg),
            TranslationError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            TranslationError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for TranslationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TranslationError::NetworkError(e) => Some(e),
            TranslationError::MaxRetriesExceeded { .. } => None,
            _ => None,
        }
    }
}

// 为常见的错误类型实现转换
impl From<reqwest::Error> for TranslationError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            TranslationError::TimeoutError
        } else if error.is_connect() {
            TranslationError::NetworkError(error)
        } else {
            TranslationError::NetworkError(error)
        }
    }
}

impl From<serde_json::Error> for TranslationError {
    fn from(error: serde_json::Error) -> Self {
        TranslationError::ServiceError(format!("JSON parsing error: {}", error))
    }
}

impl From<anyhow::Error> for TranslationError {
    fn from(error: anyhow::Error) -> Self {
        TranslationError::Other(error.to_string())
    }
}
