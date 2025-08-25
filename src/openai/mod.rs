//! OpenAI 翻译器实现

use crate::translator::Translator;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::sleep;

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

/// 用于跟踪每个API Key的使用情况
#[derive(Debug)]
struct KeyTracker {
    /// 控制并发数的信号量
    semaphore: Arc<Semaphore>,
    /// 跟踪最近的请求时间，用于RPM限制（仅在需要时使用）
    request_times: Option<Arc<Mutex<Vec<Instant>>>>,
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
    ///
    /// # 参数
    ///
    /// * `config` - OpenAI翻译器配置
    ///
    /// # 返回值
    ///
    /// 返回OpenAI翻译器实例
    pub fn new(config: OpenAIConfig) -> Self {
        let mut key_trackers = Vec::new();

        // 为每个API Key创建跟踪器
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
    fn get_system_prompt(&self, target_lang: &str) -> String {
        match &self.config.system_prompt {
            Some(prompt) => prompt.clone(),
            None => format!(
                "You are a translator. Translate the following text to {}.",
                target_lang
            ),
        }
    }

    /// 检查并等待直到可以发送请求（仅在需要时遵守RPM限制）
    ///
    /// # 参数
    ///
    /// * `tracker` - API Key跟踪器
    async fn wait_for_rate_limit(&self, tracker: &KeyTracker) {
        // 如果没有设置RPM限制，则直接返回
        let request_times = match &tracker.request_times {
            Some(times) => times,
            None => return,
        };

        let mut times = request_times.lock().await;
        let now = Instant::now();

        // 移除超过1分钟的请求记录
        times.retain(|&time| now.duration_since(time) < Duration::from_secs(60));

        // 如果请求数已达到RPM限制，则等待
        if self.config.rpm_limit > 0 && times.len() >= self.config.rpm_limit as usize {
            // 计算需要等待的时间
            if let Some(oldest) = times.first() {
                let elapsed = now.duration_since(*oldest);
                if elapsed < Duration::from_secs(60) {
                    let wait_time = Duration::from_secs(60) - elapsed;
                    drop(times); // 释放锁
                    sleep(wait_time).await;
                    // 重新获取锁并更新时间记录
                    let mut times = request_times.lock().await;
                    times.retain(|&time| now.duration_since(time) < Duration::from_secs(60));
                    times.push(Instant::now());
                    return;
                }
            }
        }

        // 记录当前请求时间（仅在需要时）
        if self.config.rpm_limit > 0 {
            times.push(now);
        }
    }
}

#[async_trait::async_trait]
impl Translator for OpenAITranslator {
    async fn translate(&self, text: &str, target_lang: &str) -> Result<String> {
        // 检查是否有配置API Key
        if self.config.api_keys.is_empty() {
            return Err(anyhow::anyhow!("No API keys configured"));
        }

        // 轮询选择API Key
        let key_index = self.get_next_key_index().await;
        let selected_key = &self.config.api_keys[key_index];
        let tracker = &self.key_trackers[key_index];

        // 获取并发许可
        let _permit = tracker.semaphore.acquire().await?;

        // 检查RPM限制（仅在需要时）
        self.wait_for_rate_limit(tracker).await;

        // 获取系统提示词
        let system_prompt = self.get_system_prompt(target_lang);

        // 构造请求
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
            temperature: 0.3,
        };

        // 发送请求
        let response = self
            .client
            .post(&format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", selected_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        // 检查HTTP状态码
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("HTTP error {}: {}", status, error_text));
        }

        // 解析响应
        #[derive(Deserialize)]
        struct Choice {
            message: Message,
        }

        #[derive(Deserialize)]
        struct Response {
            choices: Vec<Choice>,
        }

        let response_body: Response = response.json().await?;

        if response_body.choices.is_empty() {
            return Err(anyhow::anyhow!("No translation results returned"));
        }

        Ok(response_body.choices[0].message.content.clone())
    }
}

#[cfg(test)]
mod tests;
