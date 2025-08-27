//! # translate-rustdoc-example
//!
//! 这是一个展示如何使用 translate-rustdoc 库的示例程序。
//!
//! 示例展示了两种使用方式：
//! 1. 通过 TranslationManager 统一管理多个翻译器
//! 2. 直接使用单个翻译器实例

use anyhow::Result;
use async_translate::{
    LanguageIdentifier, TranslateOptions, Translator,
    manager::TranslationManager,
    microsoft::{MicrosoftConfig, MicrosoftTranslator},
    openai::{OpenAIConfig, OpenAITranslator},
};
use std::sync::Arc;
use std::time::Duration;
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
        endpoint: None, // 使用默认端点
        api_key: None,  // 使用自动认证
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

    let target_lang: LanguageIdentifier = "zh".parse().unwrap();

    info!("Translating {} texts to {}", texts.len(), target_lang);

    // 并发翻译多个文本
    let mut tasks = vec![];

    for text in texts.iter() {
        let text_str = text.to_string();
        let target_lang_clone = target_lang.clone();

        // 为每个任务创建manager的引用
        let manager1 = Arc::clone(&manager);
        let target_lang_clone1 = target_lang_clone.clone();
        let task1 = tokio::spawn(async move {
            match manager1
                .translate("openai_default", &text_str, &target_lang_clone1, None)
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
        let target_lang_clone2 = target_lang_clone.clone();
        let manager2 = Arc::clone(&manager);
        let task2 = tokio::spawn(async move {
            match manager2
                .translate("openai_custom", &text_str, &target_lang_clone2, None)
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
                .translate("microsoft", &text_str, &target_lang_clone, None)
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
        endpoint: None,      // 使用默认端点
        api_key: None,       // 使用自动认证
        concurrent_limit: 5, // 直接使用时可以设置较小的并发限制
    };
    let microsoft_translator = MicrosoftTranslator::new(microsoft_config);

    // 直接调用翻译器
    let texts = vec!["Good morning!", "How are you?", "Thank you!"];
    let target_lang: LanguageIdentifier = "zh".parse().unwrap();

    for text in texts {
        let target_lang_clone = target_lang.clone();
        match microsoft_translator
            .translate(text, &target_lang_clone, None)
            .await
        {
            Ok(result) => {
                info!("Direct translation: '{}' -> '{}'", text, result);
            }
            Err(e) => {
                error!("Direct translation error for '{}': {}", text, e);
            }
        }
    }

    info!("Direct translation example completed!");

    // 示例3：使用 LanguageIdentifier 指定源语言
    info!("\n=== 使用 LanguageIdentifier 指定源语言示例 ===");

    // 解析语言标识符
    let chinese: LanguageIdentifier = "zh-CN".parse().unwrap();
    let english: LanguageIdentifier = "en".parse().unwrap();
    let japanese: LanguageIdentifier = "ja".parse().unwrap();

    let texts = vec!["Hello", "Thank you", "Good morning"];

    for text in texts {
        let chinese_clone = chinese.clone();
        // 自动检测源语言
        match microsoft_translator
            .translate(text, &chinese_clone, None)
            .await
        {
            Ok(result) => {
                info!("Auto-detect translation: '{}' -> '{}'", text, result);
            }
            Err(e) => {
                error!("Auto-detect translation error for '{}': {}", text, e);
            }
        }
    }

    // 使用带源语言的翻译
    match microsoft_translator
        .translate("Hello", &japanese, Some(&english))
        .await
    {
        Ok(result) => {
            info!(
                "Translation with source lang: 'Hello' (en) -> '{}' (ja)",
                result
            );
        }
        Err(e) => {
            error!("Translation with source lang error: {}", e);
        }
    }

    info!("LanguageIdentifier example completed!");

    // 示例4：使用配置选项
    info!("\n=== 使用配置选项示例 ===");

    let options = TranslateOptions::default()
        .timeout(Duration::from_secs(60))
        .max_retries(5);

    match microsoft_translator
        .translate_with_options("Hello, world!", &chinese, None, &options)
        .await
    {
        Ok(result) => {
            info!("Translation with custom options: '{}'", result);
        }
        Err(e) => {
            error!("Translation with custom options error: {}", e);
        }
    }

    info!("Configuration options example completed!");

    // 示例5：批量翻译
    info!("\n=== 批量翻译示例 ===");

    let batch_texts = vec!["Hello", "World", "Rust", "Translation", "Example"];
    match microsoft_translator
        .translate_batch(&batch_texts, &chinese, None, &TranslateOptions::default())
        .await
    {
        Ok(results) => {
            info!("Batch translation results:");
            for (i, result) in results.iter().enumerate() {
                if let Some(translation) = result.translations.first() {
                    info!("  {}: '{}' -> '{}'", i, batch_texts[i], translation.text);
                    // 显示检测到的语言（如果有的话）
                    if let Some(detected) = &result.detected_language {
                        info!(
                            "    Detected language: {} (confidence: {:.2})",
                            detected.language, detected.score
                        );
                    }
                }
            }
        }
        Err(e) => {
            error!("Batch translation error: {}", e);
        }
    }

    info!("Batch translation example completed!");
    info!("All examples completed!");

    Ok(())
}
