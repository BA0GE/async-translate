//! 翻译配置选项

use std::time::Duration;

/// 翻译配置选项
#[derive(Debug, Clone)]
pub struct TranslateOptions {
    /// 请求超时时间，None 表示不超时
    pub timeout: Option<Duration>,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for TranslateOptions {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)), // 30秒超时
            max_retries: 3,                         // 重试3次
        }
    }
}

impl TranslateOptions {
    /// 设置超时时间
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 禁用超时
    pub fn no_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    /// 设置最大重试次数
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 禁用重试
    pub fn no_retries(mut self) -> Self {
        self.max_retries = 0;
        self
    }
}
