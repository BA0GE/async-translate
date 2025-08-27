//! 微软翻译器实现
//!
//! 该翻译器支持两种认证方式：
//! 1. 自动认证：通过临时token，无需配置API密钥
//! 2. API Key认证：使用用户提供的API密钥

use crate::{error::TranslationError, options::TranslateOptions, translator::Translator};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::sleep;
use unic_langid::LanguageIdentifier;

/// 微软翻译器配置
#[derive(Debug, Clone)]
pub struct MicrosoftConfig {
    /// 微软翻译服务的端点
    pub endpoint: Option<String>,
    /// API Key（可选），如果未设置则使用自动认证
    pub api_key: Option<String>,
    /// 并发请求数限制
    pub concurrent_limit: usize,
}

impl Default for MicrosoftConfig {
    fn default() -> Self {
        Self {
            endpoint: None, // 使用默认端点
            api_key: None,  // 使用自动认证
            concurrent_limit: 10,
        }
    }
}

impl MicrosoftConfig {
    pub fn builder() -> MicrosoftConfigBuilder {
        MicrosoftConfigBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct MicrosoftConfigBuilder {
    endpoint: Option<String>,
    api_key: Option<String>,
    concurrent_limit: Option<usize>,
}

impl MicrosoftConfigBuilder {
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn api_key(mut self, api_key: Option<impl Into<String>>) -> Self {
        self.api_key = api_key.map(|s| s.into());
        self
    }

    pub fn concurrent_limit(mut self, concurrent_limit: usize) -> Self {
        self.concurrent_limit = Some(concurrent_limit);
        self
    }

    pub fn build(self) -> MicrosoftConfig {
        MicrosoftConfig {
            endpoint: self.endpoint,
            api_key: self.api_key,
            concurrent_limit: self.concurrent_limit.unwrap_or(10),
        }
    }
}

/// 微软翻译器错误响应
#[derive(Debug, Deserialize)]
struct MicrosoftErrorResponse {
    error: MicrosoftErrorDetails,
}

#[derive(Debug, Deserialize)]
struct MicrosoftErrorDetails {
    code: u32,
    message: String,
}

/// 微软翻译检测到的语言信息
#[derive(Debug, Deserialize)]
pub struct DetectedLanguage {
    pub language: String,
    pub score: f64,
}

/// 微软翻译结果
#[derive(Debug, Deserialize)]
pub struct MicrosoftTranslation {
    #[serde(rename = "detectedLanguage")]
    pub detected_language: Option<DetectedLanguage>,
    pub translations: Vec<TranslationResult>,
}

/// 翻译结果
#[derive(Debug, Deserialize)]
pub struct TranslationResult {
    pub text: String,
    pub to: String,
}

/// 用于批量文本翻译的请求
#[derive(Serialize)]
struct BatchTranslationRequest {
    text: String,
}

/// 微软翻译器实现
///
/// 支持两种认证方式：
/// 1. 自动认证：通过临时token，无需配置API密钥
/// 2. API Key认证：使用用户提供的API密钥
pub struct MicrosoftTranslator {
    client: Client,
    config: MicrosoftConfig,
    semaphore: Arc<Semaphore>,
    cached_token: Arc<Mutex<Option<String>>>,
    token_expiry: Arc<Mutex<Option<Instant>>>,
}

impl MicrosoftTranslator {
    /// 创建新的微软翻译器实例
    pub fn new(config: MicrosoftConfig) -> Self {
        let concurrent_limit = config.concurrent_limit;
        Self {
            client: Client::new(),
            config,
            semaphore: Arc::new(Semaphore::new(concurrent_limit)),
            cached_token: Arc::new(Mutex::new(None)),
            token_expiry: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取认证token，带缓存和过期处理
    async fn get_auth_token(&self) -> Result<String, TranslationError> {
        // 如果配置了API Key，直接使用
        if let Some(api_key) = &self.config.api_key {
            return Ok(api_key.clone());
        }

        let mut token_guard = self.cached_token.lock().await;
        let mut expiry_guard = self.token_expiry.lock().await;

        // 检查缓存的token是否仍然有效（有效期通常为10分钟，我们提前1分钟刷新）
        if let (Some(token), Some(expiry)) = (token_guard.as_ref(), expiry_guard.as_ref()) {
            if expiry.saturating_duration_since(Instant::now()) > Duration::from_secs(60) {
                return Ok(token.clone());
            }
        }

        // 获取新的token
        let mut auth_attempts = 3;
        while auth_attempts > 0 {
            auth_attempts -= 1;
            match self.client
                .get("https://edge.microsoft.com/translate/auth")
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let token = response.text().await.map_err(|e| {
                            TranslationError::AuthenticationError(format!("Failed to read auth response: {}", e))
                        })?;
                        // 缓存新的token和过期时间
                        *token_guard = Some(token.clone());
                        *expiry_guard = Some(Instant::now() + Duration::from_secs(540)); // 9分钟后过期
                        return Ok(token);
                    } else {
                        if auth_attempts <= 0 {
                            return Err(TranslationError::AuthenticationError(
                                format!("Failed to authenticate with Microsoft Translator: HTTP {}", response.status())
                            ));
                        }
                    }
                }
                Err(e) => {
                    if auth_attempts <= 0 {
                        return Err(TranslationError::NetworkError(e));
                    }
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
        Err(TranslationError::AuthenticationError(
            "Failed to get Microsoft Translator authorization after retries".to_string(),
        ))
    }

    /// 强制清除缓存的token
    async fn clear_cached_token(&self) {
        *self.cached_token.lock().await = None;
        *self.token_expiry.lock().await = None;
    }

    /// 批量翻译文本
    ///
    /// # 参数
    ///
    /// * `texts` - 需要翻译的文本数组
    /// * `target_lang` - 目标语言标识符
    /// * `source_lang` - 源语言标识符 (None表示自动检测)
    /// * `options` - 翻译配置选项
    ///
    /// # 返回值
    ///
    /// 返回翻译结果数组
    pub async fn translate_batch(
        &self,
        texts: &[&str],
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
        options: &TranslateOptions,
    ) -> Result<Vec<MicrosoftTranslation>, TranslationError> {
        let mut errors = Vec::new();
        for attempt in 0..=options.max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(100 * 2u64.pow(attempt - 1));
                sleep(delay).await;
            }

            match self
                .try_translate_batch(texts, target_lang, source_lang, options)
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // 只在可重试的错误上继续
                    if e.is_retryable() {
                        errors.push(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Err(TranslationError::MaxRetriesExceeded {
            attempts: options.max_retries + 1,
            errors,
        })
    }

    /// 尝试批量翻译文本（无重试）
    async fn try_translate_batch(
        &self,
        texts: &[&str],
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
        options: &TranslateOptions,
    ) -> Result<Vec<MicrosoftTranslation>, TranslationError> {
        // 获取并发许可
        let _permit =
            self.semaphore.acquire().await.map_err(|e| {
                TranslationError::Other(format!("Failed to acquire semaphore: {}", e))
            })?;

        // 获取认证token
        let token = self.get_auth_token().await?;

        // 确定使用哪个端点
        let endpoint = self
            .config
            .endpoint
            .as_deref()
            .unwrap_or("https://api-edge.cognitive.microsofttranslator.com");

        // 根据超时设置创建客户端
        let client = if let Some(timeout) = options.timeout {
            Client::builder()
                .timeout(timeout)
                .build()
                .map_err(|e| TranslationError::NetworkError(e))?
        } else {
            self.client.clone()
        };

        // 构造请求
        let requests: Vec<BatchTranslationRequest> = texts
            .iter()
            .map(|text| BatchTranslationRequest {
                text: text.to_string(),
            })
            .collect();

        // 构造查询参数
        let target_lang_str = target_lang.to_string();
        let source_lang_str = source_lang.map(|s| s.to_string());
        let mut params = vec![
            ("api-version", "3.0"),
            ("to", target_lang_str.as_str()),
            ("includeSentenceLength", "true"),
        ];

        if let Some(ref source_str) = source_lang_str {
            params.push(("from", source_str.as_str()));
        }

        // 确定认证头
        let auth_header = if self.config.api_key.is_some() {
            format!("Ocp-Apim-Subscription-Key {}", token)
        } else {
            format!("Bearer {}", token)
        };

        // 发送请求
        let response = client
            .post(&format!("{}/translate", endpoint))
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .query(&params)
            .json(&requests)
            .send()
            .await?;

        // 检查HTTP状态码
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // 如果是401未授权错误，则清除缓存的token
            if status == reqwest::StatusCode::UNAUTHORIZED {
                self.clear_cached_token().await;
            }

            if let Ok(error_response) = serde_json::from_str::<MicrosoftErrorResponse>(&error_text)
            {
                return Err(TranslationError::HttpError {
                    status,
                    body: format!(
                        "Error {}: {}",
                        error_response.error.code, error_response.error.message
                    ),
                });
            }

            return Err(TranslationError::HttpError {
                status,
                body: error_text,
            });
        }

        // 解析响应
        let response_body: Vec<MicrosoftTranslation> = response.json().await?;
        Ok(response_body)
    }

    /// 翻译单个文本（公共方法）
    pub async fn translate_text(
        &self,
        text: &str,
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
        options: &TranslateOptions,
    ) -> Result<String, TranslationError> {
        let results = self
            .translate_batch(&[text], target_lang, source_lang, options)
            .await?;

        if results.is_empty() || results[0].translations.is_empty() {
            return Err(TranslationError::ServiceError(
                "No translation results returned".to_string(),
            ));
        }

        Ok(results[0].translations[0].text.clone())
    }

    /// 批量翻译文本并返回字符串数组
    pub async fn translate_batch_to_strings(
        &self,
        texts: &[&str],
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
        options: &TranslateOptions,
    ) -> Result<Vec<String>, TranslationError> {
        let results = self
            .translate_batch(texts, target_lang, source_lang, options)
            .await?;
        let translated_texts = results
            .into_iter()
            .filter_map(|res| res.translations.into_iter().next())
            .map(|trans_result| trans_result.text)
            .collect();
        Ok(translated_texts)
    }
}

#[async_trait::async_trait]
impl Translator for MicrosoftTranslator {
    async fn translate_with_options(
        &self,
        text: &str,
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
        options: &TranslateOptions,
    ) -> Result<String, TranslationError> {
        self.translate_text(text, target_lang, source_lang, options)
            .await
    }
}

#[cfg(test)]
mod tests;
