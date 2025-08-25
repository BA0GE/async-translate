//! # async-translate
//!
//! 一个支持并发的翻译库，目前支持 OpenAI 和微软翻译服务。
//!
//! ## 功能特性
//!
//! - 支持 OpenAI 翻译接口
//!   - 支持自定义 base URL、模型、API Key
//!   - 支持多 API Key 配置，每个 Key 单独计算并发数和 RPM
//!   - RPM 和并发数可选配置，具备默认数值
//! - 支持微软翻译接口
//!   - 自动获取临时认证token，无需配置API密钥
//!   - 支持并发操作
//! - 统一的翻译接口，易于扩展
//!
//! ## 使用方法
//!
//! ```rust,no_run
//! use async_translate::{TranslationManager, OpenAITranslator, OpenAIConfig, MicrosoftTranslator, MicrosoftConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 创建翻译管理器
//!     let mut manager = TranslationManager::new();
//!
//!     // 配置 OpenAI 翻译器
//!     let openai_config = OpenAIConfig {
//!         base_url: "https://api.openai.com/v1".to_string(),
//!         model: "gpt-3.5-turbo".to_string(),
//!         api_keys: vec!["your-openai-api-key".to_string()],
//!         rpm_limit: 60,
//!         concurrent_limit: 10,
//!         system_prompt: None,
//!     };
//!     let openai_translator = Box::new(OpenAITranslator::new(openai_config));
//!     manager.add_translator("openai", openai_translator);
//!
//!     // 配置微软翻译器（自动获取认证token）
//!     let microsoft_config = MicrosoftConfig {
//!         endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
//!         concurrent_limit: 10,
//!     };
//!     let microsoft_translator = Box::new(MicrosoftTranslator::new(microsoft_config));
//!     manager.add_translator("microsoft", microsoft_translator);
//!
//!     // 执行翻译
//!     let text = "Hello, world!";
//!     let target_lang = "zh";
//!
//!     let result = manager.translate("openai", text, target_lang).await?;
//!     println!("OpenAI Translation: {}", result);
//!
//!     let result = manager.translate("microsoft", text, target_lang).await?;
//!     println!("Microsoft Translation: {}", result);
//!
//!     Ok(())
//! }
//! ```

pub mod manager;
pub mod microsoft;
pub mod openai;
pub mod translator;

pub use manager::TranslationManager;
pub use microsoft::{MicrosoftConfig, MicrosoftTranslator};
pub use openai::{OpenAIConfig, OpenAITranslator};
pub use translator::Translator;

// 导出语言标识符类型
pub use unic_langid::LanguageIdentifier;
