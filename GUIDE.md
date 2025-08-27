# 使用指南

本指南将详细介绍 `async-translate` 库的安装、配置和使用方法，以及其核心功能和最佳实践。

## 快速开始

1.  在你的 `Cargo.toml` 中添加依赖：

    ```toml
    [dependencies]
    async-translate = "0.1"
    # 如果你需要使用 anyhow::Result，请添加
    anyhow = "1.0"
    # 如果你需要使用 tracing 进行日志输出，请添加
    tracing = "0.1"
    tracing-subscriber = "0.3"
    ```

2.  在代码中使用：

    最简单的启动方式是直接运行 `src/main.rs` 中的示例代码。它包含了 Microsoft 和 OpenAI 翻译器的详细用法。

    ```rust
    // 完整示例请查看 src/main.rs
    use anyhow::Result;
    use async_translate::{
        LanguageIdentifier, TranslateOptions,
        microsoft::{MicrosoftConfig, MicrosoftTranslator},
        openai::{OpenAIConfig, OpenAITranslator},
    };
    use tracing::{error, info};

    #[tokio::main]
    async fn main() -> Result<()> {
        tracing_subscriber::fmt::init();

        // Microsoft Translator 演示
        run_microsoft_translator_demo().await?;

        // OpenAI Translator 演示
        run_openai_translator_demo().await?;

        Ok(())
    }

    // run_microsoft_translator_demo 和 run_openai_translator_demo 函数定义在 src/main.rs 中
    ```

## 配置翻译器 (推荐使用 Builder 模式)

`async-translate` 库为 `OpenAIConfig` 和 `MicrosoftConfig` 提供了 Builder 模式，这是一种更符合人体工程学的配置方式，可以避免重复的 `.to_string()` 调用，并提供更清晰的默认值。

### OpenAI 配置示例

```rust
use async_translate::openai::OpenAIConfig;

let config = OpenAIConfig::builder()
    .api_keys(vec!["your-openai-api-key"]) // 支持 Vec<&str> 或 Vec<String>
    .base_url("https://api.openai.com/v1") // 默认值: "https://api.openai.com/v1"
    .model("gpt-3.5-turbo") // 默认值: "gpt-3.5-turbo"
    .rpm_limit(60) // 默认值: 60 (每分钟请求数限制)
    .concurrent_limit(10) // 默认值: 10 (并发请求数限制)
    .system_prompt("You are a helpful assistant.") // 默认值: None (使用库内置的优化提示词)
    .build();
```

### 微软配置示例

```rust
use async_translate::microsoft::MicrosoftConfig;

let config = MicrosoftConfig::builder()
    .api_key(Some("your-microsoft-api-key")) // 默认值: None (表示自动认证)
    .endpoint("https://api-edge.cognitive.microsofttranslator.com") // 默认值: None (使用库内置的默认端点)
    .concurrent_limit(10) // 默认值: 10 (并发请求数限制)
    .build();
```

## 核心功能

### 1. 单个文本翻译

使用 `translate_text` 方法翻译单个字符串。

```rust
use async_translate::{
    LanguageIdentifier, TranslateOptions,
    microsoft::{MicrosoftConfig, MicrosoftTranslator},
};
use std::time::Duration;

// 创建翻译器实例 (使用 Builder 模式)
let translator = MicrosoftTranslator::new(MicrosoftConfig::builder().build());

let text_to_translate = "Hello, world!";
let target_lang: LanguageIdentifier = "zh-Hans".parse().unwrap();

match translator.translate_text(text_to_translate, &target_lang, None, &TranslateOptions::default()).await {
    Ok(result) => println!("'{}' -> '{}'", text_to_translate, result),
    Err(e) => eprintln!("单文本翻译失败: {}", e),
}
```

### 2. 批量翻译

库提供了两种批量翻译方法：

*   `translate_batch_to_strings` (推荐用于微软翻译): 直接返回 `Vec<String>`，简化结果处理。
*   `translate_batch`: 返回包含更多详情的 `Vec<MicrosoftTranslation>` 或 `Vec<String>` (OpenAI)，适用于需要检测语言等额外信息的场景。

```rust
use async_translate::{
    LanguageIdentifier, TranslateOptions,
    microsoft::{MicrosoftConfig, MicrosoftTranslator},
};

// 创建翻译器实例
let translator = MicrosoftTranslator::new(MicrosoftConfig::builder().build());

let texts_to_translate = vec!["Hello", "World", "Rust is amazing"];
let target_lang: LanguageIdentifier = "zh-Hans".parse().unwrap();

// 批量翻译 (简化结果)
match translator.translate_batch_to_strings(&texts_to_translate, &target_lang, None, &TranslateOptions::default()).await {
    Ok(results) => {
        println!("批量翻译结果:");
        for (original, translated) in texts_to_translate.iter().zip(results.iter()) {
            println!("  '{}' -> '{}'", original, translated);
        }
    },
    Err(e) => eprintln!("批量翻译失败: {}", e),
}

// 批量翻译 (完整结果 - 仅适用于微软翻译)
// 对于 OpenAI，translate_batch 也直接返回 Vec<String>
match translator.translate_batch(&texts_to_translate, &target_lang, None, &TranslateOptions::default()).await {
    Ok(results) => {
        println!("批量翻译结果 (带详情):");
        // 微软翻译返回 Vec<MicrosoftTranslation>
        // OpenAI 翻译返回 Vec<String>
        // 根据实际类型处理结果
    },
    Err(e) => eprintln!("批量翻译失败: {}", e),
}
```

### 3. 指定源语言翻译

```rust
use async_translate::{
    LanguageIdentifier, TranslateOptions,
    microsoft::{MicrosoftConfig, MicrosoftTranslator},
};

// 创建翻译器实例
let translator = MicrosoftTranslator::new(MicrosoftConfig::builder().build());

let text_with_source = "This text is definitely English.";
let target_lang: LanguageIdentifier = "zh-Hans".parse().unwrap();
let source_lang: LanguageIdentifier = "en".parse().unwrap();

match translator.translate_text(text_with_source, &target_lang, Some(&source_lang), &TranslateOptions::default()).await {
    Ok(result) => println!("'{}' (from {}) -> '{}'", text_with_source, source_lang, result),
    Err(e) => eprintln!("指定源语言翻译失败: {}", e),
}
```

### 4. 自定义选项 (超时和重试)

`TranslateOptions` 允许您配置请求的超时时间和最大重试次数。

```rust
use async_translate::{
    LanguageIdentifier, TranslateOptions,
    microsoft::{MicrosoftConfig, MicrosoftTranslator},
};
use std::time::Duration;

// 创建翻译器实例
let translator = MicrosoftTranslator::new(MicrosoftConfig::builder().build());

let target_lang: LanguageIdentifier = "zh-Hans".parse().unwrap();

let options = TranslateOptions::default()
    .timeout(Duration::from_secs(30))
    .max_retries(2);

match translator.translate_text("Testing custom options", &target_lang, None, &options).await {
    Ok(result) => println!("自定义选项翻译成功: '{}'", result),
    Err(e) => eprintln!("自定义选项翻译失败: {}", e),
}
```

## 语言标识符 (LanguageIdentifier)

库使用 `unic-langid` 库提供类型安全的语言定义，符合 BCP 47 和 Unicode 标准。

### 优势

*   **类型安全**：编译时检查语言代码的有效性，避免运行时错误。
*   **IDE 支持**：智能提示和自动补全，提高开发效率。
*   **标准兼容**：确保语言代码的规范性和互操作性。

### 常用语言标识符

| 语言 | 标识符 |
|------|--------|
| 中文（简体） | `"zh-Hans"` |
| 中文（繁体） | `"zh-Hant"` |
| 英文 | `"en"` |
| 日文 | `"ja"` |
| 韩文 | `"ko"` |
| 法文 | `"fr"` |
| 德文 | `"de"` |
| 西班牙文 | `"es"` |

## 错误处理

库使用 `TranslationError` 枚举来表示各种翻译过程中可能发生的错误，例如网络错误、HTTP 错误、认证错误等。所有错误都实现了 `std::error::Error` 和 `std::fmt::Display`。

## 并发与性能

`async-translate` 库天生支持异步并发操作，并通过内部信号量机制限制并发请求数和 RPM (Requests Per Minute)，确保对外部 API 的友好访问。

*   **并发限制**：每个翻译器实例独立管理其并发限制。
*   **令牌缓存**：微软翻译器实现了认证令牌的缓存机制，减少了不必要的网络请求，提高了性能。
*   **OpenAI 提示优化**：通过精细的提示工程，确保 OpenAI 翻译器只返回纯净的翻译结果，减少了不必要的 token 消耗。

## 许可证

本项目采用 MIT 许可证。查看 [LICENSE](LICENSE) 文件了解更多信息。
