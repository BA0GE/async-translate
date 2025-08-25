//! # translate-rustdoc-example
//!
//! 这是一个展示如何使用 translate-rustdoc 库的示例程序。
//!
//! 示例展示了两种使用方式：
//! 1. 通过 TranslationManager 统一管理多个翻译器
//! 2. 直接使用单个翻译器实例

use anyhow::Result;
use async_translate::{
    manager::TranslationManager,
    microsoft::{MicrosoftConfig, MicrosoftTranslator},
    openai::{OpenAIConfig, OpenAITranslator},
    LanguageIdentifier, Translator,
};
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 创建翻译管理器
    let mut manager = TranslationManager::new();

    // 配置OpenAI翻译器（使用默认提示词）
    let openai_config_default = OpenAIConfig {
        base_url: "https://api.openai.com/v1".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        api_keys: vec!["your-first-openai-api-key".to_string()],
        rpm_limit: 60,
        concurrent_limit: 10,
        system_prompt: None, // 使用默认提示词
    };
    let openai_translator_default = Box::new(OpenAITranslator::new(openai_config_default));
    manager.add_translator("openai_default", openai_translator_default);

    // 配置OpenAI翻译器（使用自定义提示词）
    let openai_config_custom = OpenAIConfig {
        base_url: "https://api.openai.com/v1".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        api_keys: vec![
            "your-second-openai-api-key".to_string(),
        ],
        rpm_limit: 60,
        concurrent_limit: 10,
        system_prompt: Some("You are a professional translator with expertise in technical documentation. Please translate the following text to high-quality {target_lang} while preserving technical accuracy and context.".to_string()),
    };
    let openai_translator_custom = Box::new(OpenAITranslator::new(openai_config_custom));
    manager.add_translator("openai_custom", openai_translator_custom);

    // 配置微软翻译器（自动获取认证token）
    let microsoft_config = MicrosoftConfig {
        endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
        concurrent_limit: 10,
    };
    let microsoft_translator = Box::new(MicrosoftTranslator::new(microsoft_config));
    manager.add_translator("microsoft", microsoft_translator);

    // 将manager包装成Arc以在线程间共享
    let manager = Arc::new(manager);

    // 测试翻译
    let texts = vec![
        "Hello, world!",
        "How are you today?",
        "The weather is nice.",
        "I'm learning Rust programming language.",
        "This is a concurrent translation example.",
    ];

    let target_lang = "zh";

    info!("Translating {} texts to {}", texts.len(), target_lang);

    // 并发翻译多个文本
    let mut tasks = vec![];

    for text in texts.iter() {
        let text_str = text.to_string();

        // 为每个任务创建manager的引用
        let manager1 = Arc::clone(&manager);
        let task1 = tokio::spawn(async move {
            match manager1
                .translate("openai_default", &text_str, &target_lang)
                .await
            {
                Ok(result) => {
                    info!(
                        "OpenAI (default) Translation: '{}' -> '{}'",
                        text_str, result
                    );
                    Some(result)
                }
                Err(e) => {
                    error!(
                        "OpenAI (default) Translation error for '{}': {}",
                        text_str, e
                    );
                    None
                }
            }
        });
        tasks.push(task1);

        let text_str = text.to_string();
        let manager2 = Arc::clone(&manager);
        let task2 = tokio::spawn(async move {
            match manager2
                .translate("openai_custom", &text_str, &target_lang)
                .await
            {
                Ok(result) => {
                    info!(
                        "OpenAI (custom) Translation: '{}' -> '{}'",
                        text_str, result
                    );
                    Some(result)
                }
                Err(e) => {
                    error!(
                        "OpenAI (custom) Translation error for '{}': {}",
                        text_str, e
                    );
                    None
                }
            }
        });
        tasks.push(task2);

        let text_str = text.to_string();
        let manager3 = Arc::clone(&manager);
        let task3 = tokio::spawn(async move {
            match manager3
                .translate("microsoft", &text_str, &target_lang)
                .await
            {
                Ok(result) => {
                    info!("Microsoft Translation: '{}' -> '{}'", text_str, result);
                    Some(result)
                }
                Err(e) => {
                    error!("Microsoft Translation error for '{}': {}", text_str, e);
                    None
                }
            }
        });
        tasks.push(task3);
    }

    // 等待所有翻译任务完成
    for task in tasks {
        if let Err(e) = task.await {
            error!("Task failed: {}", e);
        }
    }

    info!("All translations completed!");

    // 示例2：直接使用翻译器实例
    info!("\n=== 直接使用翻译器实例示例 ===");

    let microsoft_config = MicrosoftConfig {
        endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
        concurrent_limit: 5, // 直接使用时可以设置较小的并发限制
    };
    let microsoft_translator = MicrosoftTranslator::new(microsoft_config);

    // 直接调用翻译器
    let texts = vec!["Good morning!", "How are you?", "Thank you!"];

    for text in texts {
        match microsoft_translator.translate(text, "zh").await {
            Ok(result) => {
                info!("Direct translation: '{}' -> '{}'", text, result);
            }
            Err(e) => {
                error!("Direct translation error for '{}': {}", text, e);
            }
        }
    }

    info!("Direct translation example completed!");

    // 示例3：使用 LanguageIdentifier
    info!("\n=== 使用 LanguageIdentifier 示例 ===");

    // LanguageIdentifier 已在文件顶部导入

    // 解析语言标识符
    let chinese: LanguageIdentifier = "zh-CN".parse().unwrap();
    let english: LanguageIdentifier = "en".parse().unwrap();
    let japanese: LanguageIdentifier = "ja".parse().unwrap();

    let texts = vec!["Hello", "Thank you", "Good morning"];

    for text in texts {
        match microsoft_translator.translate_langid(text, &chinese).await {
            Ok(result) => {
                info!("LanguageIdentifier translation: '{}' -> '{}'", text, result);
            }
            Err(e) => {
                error!("LanguageIdentifier translation error for '{}': {}", text, e);
            }
        }
    }

    // 使用带源语言的翻译
    match microsoft_translator.translate_with_langid("Hello", Some(&english), &japanese).await {
        Ok(result) => {
            info!("Translation with source lang: 'Hello' (en) -> '{}' (ja)", result);
        }
        Err(e) => {
            error!("Translation with source lang error: {}", e);
        }
    }

    info!("LanguageIdentifier example completed!");

    Ok(())
}
