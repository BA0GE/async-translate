//! OpenAI 翻译器实现

use crate::{error::TranslationError, options::TranslateOptions, translator::Translator};
use futures::future::join_all;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::sleep;
use unic_langid::LanguageIdentifier;

/// OpenAI翻译器配置
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    /// OpenAI API 的基础 URL
    pub base_url: String,
    /// 使用的模型名称
    pub model: String,
    /// API Key 列表，支持多个 Key
    pub api_keys: Vec<String>,
    /// 每分钟请求数限制，设为0表示不限制
    pub rpm_limit: u32,
    /// 并发请求数限制
    pub concurrent_limit: usize,
    /// 自定义系统提示词，如果为None则使用默认提示词
    pub system_prompt: Option<String>,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_keys: vec![],
            rpm_limit: 60,
            concurrent_limit: 10,
            system_prompt: None,
        }
    }
}

impl OpenAIConfig {
    pub fn builder() -> OpenAIConfigBuilder {
        OpenAIConfigBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct OpenAIConfigBuilder {
    base_url: Option<String>,
    model: Option<String>,
    api_keys: Option<Vec<String>>,
    rpm_limit: Option<u32>,
    concurrent_limit: Option<usize>,
    system_prompt: Option<String>,
}

impl OpenAIConfigBuilder {
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn api_keys(mut self, api_keys: Vec<impl Into<String>>) -> Self {
        self.api_keys = Some(api_keys.into_iter().map(|s| s.into()).collect());
        self
    }

    pub fn rpm_limit(mut self, rpm_limit: u32) -> Self {
        self.rpm_limit = Some(rpm_limit);
        self
    }

    pub fn concurrent_limit(mut self, concurrent_limit: usize) -> Self {
        self.concurrent_limit = Some(concurrent_limit);
        self
    }

    pub fn system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    pub fn build(self) -> OpenAIConfig {
        OpenAIConfig {
            base_url: self
                .base_url
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model: self.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            api_keys: self.api_keys.unwrap_or_else(Vec::new),
            rpm_limit: self.rpm_limit.unwrap_or(60),
            concurrent_limit: self.concurrent_limit.unwrap_or(10),
            system_prompt: self.system_prompt,
        }
    }
}

/// 用于跟踪每个API Key的使用情况
#[derive(Debug)]
struct KeyTracker {
    /// 控制并发数的信号量
    semaphore: Arc<Semaphore>,
    /// 跟踪最近的请求时间，用于RPM限制（仅在需要时使用）
    request_times: Option<Arc<Mutex<Vec<Instant>>>>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct Request {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Response {
    choices: Vec<Choice>,
}

/// OpenAI翻译器实现
pub struct OpenAITranslator {
    client: Client,
    config: OpenAIConfig,
    /// 每个API Key对应的跟踪器
    key_trackers: Arc<Vec<KeyTracker>>,
    /// 用于轮询选择API Key的索引
    current_key_index: Arc<Mutex<usize>>,
}

impl OpenAITranslator {
    /// 创建新的OpenAI翻译器实例
    pub fn new(config: OpenAIConfig) -> Self {
        let mut key_trackers = Vec::new();
        for _ in &config.api_keys {
            let request_times = if config.rpm_limit > 0 {
                Some(Arc::new(Mutex::new(Vec::new())))
            } else {
                None
            };
            key_trackers.push(KeyTracker {
                semaphore: Arc::new(Semaphore::new(config.concurrent_limit)),
                request_times,
            });
        }
        Self {
            client: Client::new(),
            config,
            key_trackers: Arc::new(key_trackers),
            current_key_index: Arc::new(Mutex::new(0)),
        }
    }

    /// 轮询选择下一个可用的API Key索引
    async fn get_next_key_index(&self) -> usize {
        let mut index = self.current_key_index.lock().await;
        let current = *index;
        *index = (*index + 1) % self.config.api_keys.len();
        current
    }

    /// 获取系统提示词
    fn get_system_prompt(&self, target_lang: &str, source_lang: Option<&str>) -> String {
        if let Some(prompt) = &self.config.system_prompt {
            return prompt.clone();
        }
        let source_lang_str = source_lang.unwrap_or("auto");
        format!(
            "You are a raw translation engine. You are not an AI assistant. Your only function is to translate the user's text. Translate from {} to {}. Do not, under any circumstances, write anything other than the translated text. Do not apologize. Do not explain. Do not add any extra text. If you cannot translate the text, repeat the original text.\n\nExamples:\n\nUser: Hello\nAssistant: 你好\n\nUser: World\nAssistant: 世界\n\nUser: xyzabc\nAssistant: xyzabc",
            source_lang_str, target_lang
        )
    }

    /// 检查并等待直到可以发送请求（遵守RPM限制）
    async fn wait_for_rate_limit(&self, tracker: &KeyTracker) {
        if let Some(request_times) = &tracker.request_times {
            let mut times = request_times.lock().await;
            let now = Instant::now();
            times.retain(|&time| now.duration_since(time) < Duration::from_secs(60));
            if self.config.rpm_limit > 0 && times.len() >= self.config.rpm_limit as usize {
                if let Some(oldest) = times.first() {
                    let elapsed = now.duration_since(*oldest);
                    if elapsed < Duration::from_secs(60) {
                        sleep(Duration::from_secs(60) - elapsed).await;
                    }
                }
            }
            times.push(now);
        }
    }

    /// 批量翻译文本
    pub async fn translate_batch(
        &self,
        texts: &[&str],
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
        options: &TranslateOptions,
    ) -> Result<Vec<String>, TranslationError> {
        let mut futures = Vec::new();
        for &text in texts {
            let future = self.translate_text_with_retry(text, target_lang, source_lang, options);
            futures.push(future);
        }
        let results: Vec<_> = join_all(futures).await;
        results.into_iter().collect()
    }

    /// 使用重试逻辑翻译单个文本
    async fn translate_text_with_retry(
        &self,
        text: &str,
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
        options: &TranslateOptions,
    ) -> Result<String, TranslationError> {
        let mut errors = Vec::new();
        for attempt in 0..=options.max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(100 * 2u64.pow(attempt - 1));
                sleep(delay).await;
            }
            match self
                .try_translate_single(text, target_lang, source_lang, options)
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
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

    /// 尝试翻译单个文本（无重试）
    async fn try_translate_single(
        &self,
        text: &str,
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
        options: &TranslateOptions,
    ) -> Result<String, TranslationError> {
        if self.config.api_keys.is_empty() {
            return Err(TranslationError::ConfigurationError(
                "No API keys configured".to_string(),
            ));
        }

        let key_index = self.get_next_key_index().await;
        let selected_key = &self.config.api_keys[key_index];
        let tracker = &self.key_trackers[key_index];

        let _permit =
            tracker.semaphore.acquire().await.map_err(|e| {
                TranslationError::Other(format!("Failed to acquire semaphore: {}", e))
            })?;
        self.wait_for_rate_limit(tracker).await;

        let client = if let Some(timeout) = options.timeout {
            Client::builder()
                .timeout(timeout)
                .build()
                .map_err(|e| TranslationError::NetworkError(e))?
        } else {
            self.client.clone()
        };

        let source_lang_str = source_lang.map(|s| s.to_string());
        let system_prompt =
            self.get_system_prompt(&target_lang.to_string(), source_lang_str.as_deref());

        let request = Request {
            model: self.config.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: text.to_string(),
                },
            ],
            temperature: 0.0,
        };

        let response = client
            .post(&format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", selected_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TranslationError::HttpError { status, body });
        }

        let response_body: Response = response.json().await?;
        response_body
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| {
                TranslationError::ServiceError("No translation results returned".to_string())
            })
    }

    /// 翻译单个文本
    pub async fn translate_text(
        &self,
        text: &str,
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
        options: &TranslateOptions,
    ) -> Result<String, TranslationError> {
        self.translate_text_with_retry(text, target_lang, source_lang, options)
            .await
    }
}

#[async_trait::async_trait]
impl Translator for OpenAITranslator {
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
