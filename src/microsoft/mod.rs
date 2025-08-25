//! 微软翻译器实现
//!
//! 该翻译器使用自动获取的临时认证token，无需手动配置API密钥。
//! 每次翻译时会自动从Microsoft的边缘服务获取临时token。

use crate::translator::Translator;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// 微软翻译器配置
///
/// 该配置用于设置微软翻译器的参数。由于使用自动认证方式，
/// 无需配置API密钥，系统会自动获取临时认证token。
#[derive(Debug, Clone)]
pub struct MicrosoftConfig {
    /// 微软翻译服务的端点
    pub endpoint: String,
    /// 并发请求数限制
    pub concurrent_limit: usize,
}

impl Default for MicrosoftConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
            concurrent_limit: 10,
        }
    }
}

/// 微软翻译器实现
///
/// 使用自动认证方式，通过临时token进行API调用。
/// 不需要手动管理API密钥，系统会自动获取和刷新认证token。
pub struct MicrosoftTranslator {
    client: Client,
    config: MicrosoftConfig,
    /// 控制并发数的信号量
    semaphore: Arc<Semaphore>,
    /// 认证token
    auth: Arc<Mutex<Option<String>>>,
}

impl MicrosoftTranslator {
    /// 创建新的微软翻译器实例
    ///
    /// 该翻译器使用自动认证方式，无需配置API密钥。
    ///
    /// # 参数
    ///
    /// * `config` - 微软翻译器配置（包含端点和并发限制）
    ///
    /// # 返回值
    ///
    /// 返回配置好的微软翻译器实例
    pub fn new(config: MicrosoftConfig) -> Self {
        Self {
            client: Client::new(),
            config: config.clone(),
            semaphore: Arc::new(Semaphore::new(config.concurrent_limit)),
            auth: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取认证token
    ///
    /// 通过访问Microsoft的边缘认证服务自动获取临时token。
    /// 该token具有一定的有效期，系统会自动管理token的获取和缓存。
    ///
    /// # 返回值
    ///
    /// 返回获取到的Bearer token字符串
    async fn authenticate(&self) -> Result<String> {
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
                        let token = response.text().await?;
                        return Ok(token);
                    } else {
                        if auth_attempts <= 0 {
                            return Err(anyhow::anyhow!("Failed to authenticate with Microsoft Translator: HTTP {}", response.status()));
                        }
                    }
                }
                Err(e) => {
                    if auth_attempts <= 0 {
                        return Err(anyhow::anyhow!("Failed to connect to Microsoft Translator: {}", e));
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
        Err(anyhow::anyhow!("Failed to get Microsoft Translator authorization after retries"))
    }
}

#[async_trait::async_trait]
impl Translator for MicrosoftTranslator {
    async fn translate(&self, text: &str, target_lang: &str) -> Result<String> {
        // 获取并发许可
        let _permit = self.semaphore.acquire().await?;

        // 获取认证token
        let token = {
            let mut auth_guard = self.auth.lock().await;
            if auth_guard.is_none() {
                *auth_guard = Some(self.authenticate().await?);
            }
            auth_guard.clone().unwrap()
        };

        // 构造请求
        #[derive(Serialize)]
        struct Request {
            text: String,
        }

        let request = vec![Request {
            text: text.to_string(),
        }];

        // 构造查询参数
        let params = [
            ("api-version", "3.0"),
            ("to", target_lang),
            ("includeSentenceLength", "true"),
        ];

        // 发送请求
        let response = self.client
            .post(&format!("{}/translate", self.config.endpoint))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .query(&params)
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
        struct Translation {
            text: String,
        }

        #[derive(Deserialize)]
        struct ResponseItem {
            translations: Vec<Translation>,
        }

        let response_body: Vec<ResponseItem> = response.json().await?;

        if response_body.is_empty() || response_body[0].translations.is_empty() {
            return Err(anyhow::anyhow!("No translation results returned"));
        }

        Ok(response_body[0].translations[0].text.clone())
    }
}

#[cfg(test)]
mod tests;