use anyhow::Result;
use async_translate::{
    LanguageIdentifier, TranslateOptions,
    microsoft::{MicrosoftConfig, MicrosoftTranslator},
    openai::{OpenAIConfig, OpenAITranslator},
};
use std::time::Duration;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // --- Microsoft Translator Demonstration ---
    run_microsoft_translator_demo().await?;

    // --- OpenAI Translator Demonstration ---
    run_openai_translator_demo().await?;

    Ok(())
}

async fn run_microsoft_translator_demo() -> Result<()> {
    info!("\n=== Microsoft Translator Demonstration ===");

    // 1. 创建翻译器实例
    // 使用自动认证，无需API Key
    let config = MicrosoftConfig::builder()
        .api_key(None::<String>) // 使用自动认证
        .build();
    let translator = MicrosoftTranslator::new(config);

    // 2. 定义翻译语言
    let target_lang: LanguageIdentifier = "zh-Hans".parse()?;
    let source_lang: LanguageIdentifier = "en".parse()?;

    // 3. 翻译单个文本
    info!("\n--- 场景1: 翻译单个文本 ---");
    let text_to_translate = "Hello, world!";
    match translator
        .translate_text(
            text_to_translate,
            &target_lang,
            None,
            &TranslateOptions::default(),
        )
        .await
    {
        Ok(result) => info!("'{}' -> '{}'", text_to_translate, result),
        Err(e) => error!("单文本翻译失败: {}", e),
    }

    // 4. 批量翻译 (使用简化的 `translate_batch_to_strings`)
    info!("\n--- 场景2: 批量翻译 (简化结果) ---");
    let texts_to_translate = vec!["Hello", "World", "Rust is amazing"];
    match translator
        .translate_batch_to_strings(
            &texts_to_translate,
            &target_lang,
            None,
            &TranslateOptions::default(),
        )
        .await
    {
        Ok(results) => {
            info!("批量翻译结果:");
            for (original, translated) in texts_to_translate.iter().zip(results.iter()) {
                info!("  '{}' -> '{}'", original, translated);
            }
        }
        Err(e) => error!("批量翻译失败: {}", e),
    }

    // 5. 批量翻译 (使用 `translate_batch` 获取完整信息)
    info!("\n--- 场景3: 批量翻译 (完整结果) ---");
    match translator
        .translate_batch(
            &texts_to_translate,
            &target_lang,
            None,
            &TranslateOptions::default(),
        )
        .await
    {
        Ok(results) => {
            info!("批量翻译结果 (带详情):");
            for result in results {
                if let Some(translation) = result.translations.first() {
                    info!("  翻译: '{}'", translation.text);
                    if let Some(detected) = result.detected_language {
                        info!(
                            "    -> 检测到源语言: {} (置信度: {})",
                            detected.language, detected.score
                        );
                    }
                }
            }
        }
        Err(e) => error!("批量翻译失败: {}", e),
    }

    // 6. 指定源语言进行翻译
    info!("\n--- 场景4: 指定源语言翻译 ---");
    let text_with_source = "This text is definitely English.";
    match translator
        .translate_text(
            text_with_source,
            &target_lang,
            Some(&source_lang),
            &TranslateOptions::default(),
        )
        .await
    {
        Ok(result) => info!(
            "'{}' (from {}) -> '{}'",
            text_with_source, source_lang, result
        ),
        Err(e) => error!("指定源语言翻译失败: {}", e),
    }

    // 7. 使用自定义选项（例如，超时和重试）
    info!("\n--- 场景5: 自定义选项 (超时和重试) ---");
    let options = TranslateOptions::default()
        .timeout(Duration::from_secs(30))
        .max_retries(2);
    match translator
        .translate_text("Testing custom options", &target_lang, None, &options)
        .await
    {
        Ok(result) => info!("自定义选项翻译成功: '{}'", result),
        Err(e) => error!("自定义选项翻译失败: {}", e),
    }

    Ok(())
}

async fn run_openai_translator_demo() -> Result<()> {
    info!("\n=== OpenAI Translator Demonstration ===");

    // 1. 创建翻译器实例 (请替换为你的API Key)
    let config = OpenAIConfig::builder()
        .api_keys(vec!["your-openai-api-key"])
        .base_url("https://api.openai.com/v1")
        .model("gpt-3.5-turbo")
        .build();
    let translator = OpenAITranslator::new(config);
    let target_lang: LanguageIdentifier = "zh-Hans".parse()?;
    let source_lang: LanguageIdentifier = "en".parse()?;

    info!("\n--- 场景1: 翻译单个文本 ---");
    let text_to_translate = "Hello, AI!";
    match translator
        .translate_text(
            text_to_translate,
            &target_lang,
            None,
            &TranslateOptions::default(),
        )
        .await
    {
        Ok(result) => info!("'{}' -> '{}'", text_to_translate, result),
        Err(e) => error!("单文本翻译失败: {}", e),
    }

    info!("\n--- 场景2: 批量翻译 ---");
    let texts_to_translate = vec!["Hello again", "This is a batch test", "AI is powerful"];
    match translator
        .translate_batch(
            &texts_to_translate,
            &target_lang,
            None,
            &TranslateOptions::default(),
        )
        .await
    {
        Ok(results) => {
            info!("批量翻译结果:");
            for (original, translated) in texts_to_translate.iter().zip(results.iter()) {
                info!("  '{}' -> '{}'", original, translated);
            }
        }
        Err(e) => error!("批量翻译失败: {}", e),
    }

    info!("\n--- 场景3: 指定源语言翻译 ---");
    let text_with_source = "This text is definitely English.";
    match translator
        .translate_text(
            text_with_source,
            &target_lang,
            Some(&source_lang),
            &TranslateOptions::default(),
        )
        .await
    {
        Ok(result) => info!(
            "'{}' (from {}) -> '{}'",
            text_with_source, source_lang, result
        ),
        Err(e) => error!("指定源语言翻译失败: {}", e),
    }

    Ok(())
}
