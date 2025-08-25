//! # LanguageIdentifier 示例
//!
//! 展示如何使用 unic-langid 库进行类型安全的语言定义

use anyhow::Result;
use async_translate::{
    manager::TranslationManager,
    microsoft::{MicrosoftConfig, MicrosoftTranslator},
    LanguageIdentifier, Translator,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🗣️  LanguageIdentifier 使用示例\n");

    // 创建翻译器
    let translator = MicrosoftTranslator::new(MicrosoftConfig {
        endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
        concurrent_limit: 10,
    });

    // 解析语言标识符
    let chinese: LanguageIdentifier = "zh-CN".parse().unwrap();
    let english: LanguageIdentifier = "en".parse().unwrap();
    let japanese: LanguageIdentifier = "ja".parse().unwrap();
    let korean: LanguageIdentifier = "ko".parse().unwrap();

    println!("📝 基本翻译示例：");
    println!("   英文 'Hello' -> 中文 '{}'", chinese);

    match translator.translate_langid("Hello", &chinese).await {
        Ok(result) => println!("   结果: '{}'\n", result),
        Err(e) => println!("   错误: {}\n", e),
    }

    println!("🔄 带源语言的翻译示例：");
    println!("   英文 'Hello' -> 日文 '{}'", japanese);

    match translator.translate_with_langid("Hello", Some(&english), &japanese).await {
        Ok(result) => println!("   结果: '{}'\n", result),
        Err(e) => println!("   错误: {}\n", e),
    }

    println!("🌐 多语言翻译示例：");
    let text = "Thank you";
    let languages = vec![
        ("中文", &chinese),
        ("日文", &japanese),
        ("韩文", &korean),
    ];

    for (lang_name, lang_id) in languages {
        match translator.translate_langid(text, lang_id).await {
            Ok(result) => println!("   英文 '{}' -> {} '{}'", text, lang_name, result),
            Err(e) => println!("   英文 '{}' -> {} 错误: {}", text, lang_name, e),
        }
    }

    println!("\n🎯 Manager 中使用 LanguageIdentifier：");
    let mut manager = TranslationManager::new();
    manager.add_translator("microsoft", Box::new(MicrosoftTranslator::new(MicrosoftConfig {
        endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
        concurrent_limit: 5,
    })));

    match manager.translate_langid("microsoft", "Good morning", &chinese).await {
        Ok(result) => println!("   Manager 翻译结果: '{}'", result),
        Err(e) => println!("   Manager 翻译错误: {}", e),
    }

    println!("\n✅ LanguageIdentifier 示例完成！");
    println!("\n💡 LanguageIdentifier 的优势：");
    println!("   • 类型安全 - 编译时检查语言代码有效性");
    println!("   • IDE 支持 - 智能提示和自动补全");
    println!("   • 标准兼容 - 符合 BCP 47 和 Unicode 标准");
    println!("   • 错误减少 - 避免拼写错误和无效语言代码");

    Ok(())
}