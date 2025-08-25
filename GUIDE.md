# 使用指南

## 快速开始

1. 在你的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
async-translate = "0.1"
```

2. 在代码中使用：

```rust
use anyhow::Result;
use async_translate::{
    manager::TranslationManager,
    openai::{OpenAITranslator, OpenAIConfig},
    microsoft::{MicrosoftTranslator, MicrosoftConfig},
};

#[tokio::main]
async fn main() -> Result<()> {
    // 创建翻译管理器
    let mut manager = TranslationManager::new();
    
    // 配置 OpenAI 翻译器（使用默认提示词）
    let openai_config = OpenAIConfig::default();
    let openai_translator = Box::new(OpenAITranslator::new(openai_config));
    manager.add_translator("openai", openai_translator);
    
    // 执行翻译
    let text = "Hello, world!";
    let target_lang = "zh";
    
    let result = manager.translate("openai", text, target_lang).await?;
    println!("Translation: {}", result);
    
    Ok(())
}
```

## 配置 OpenAI 翻译器

### 使用默认提示词

```rust
let openai_config = OpenAIConfig {
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-3.5-turbo".to_string(),
    api_keys: vec!["your-api-key".to_string()],
    rpm_limit: 60,
    concurrent_limit: 10,
    system_prompt: None, // 使用默认提示词
};

let openai_translator = Box::new(OpenAITranslator::new(openai_config));
```

### 使用自定义提示词

```rust
let openai_config = OpenAIConfig {
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-3.5-turbo".to_string(),
    api_keys: vec!["your-api-key".to_string()],
    rpm_limit: 60,
    concurrent_limit: 10,
    system_prompt: Some("You are a professional translator with expertise in technical documentation. Please translate the following text to high-quality {target_lang} while preserving technical accuracy.".to_string()),
};

let openai_translator = Box::new(OpenAITranslator::new(openai_config));
```

## 配置微软翻译器

```rust
let microsoft_config = MicrosoftConfig {
    endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
    concurrent_limit: 10,
};

let microsoft_translator = Box::new(MicrosoftTranslator::new(microsoft_config));
```

## 多 API Key 支持

```rust
let openai_config = OpenAIConfig {
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-3.5-turbo".to_string(),
    api_keys: vec![
        "api-key-1".to_string(),
        "api-key-2".to_string(),
        "api-key-3".to_string(),
    ],
    rpm_limit: 60,
    concurrent_limit: 10,
    system_prompt: None,
};
```

## 不限制 RPM

```rust
let openai_config = OpenAIConfig {
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-3.5-turbo".to_string(),
    api_keys: vec!["your-api-key".to_string()],
    rpm_limit: 0, // 不限制 RPM
    concurrent_limit: 10,
    system_prompt: None,
};
```

## 直接使用翻译器

除了通过 TranslationManager，你还可以直接使用翻译器实例：

### 直接使用微软翻译器

```rust
use anyhow::Result;
use async_translate::{microsoft::{MicrosoftConfig, MicrosoftTranslator}, Translator};

#[tokio::main]
async fn main() -> Result<()> {
    // 配置微软翻译器（自动获取认证token）
    let microsoft_config = MicrosoftConfig {
        endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
        concurrent_limit: 10,
    };

    let translator = MicrosoftTranslator::new(microsoft_config);

    // 直接调用翻译
    let result = translator.translate("Hello, world!", "zh").await?;
    println!("Translation: {}", result); // 输出: 世界您好！

    Ok(())
}
```

### 使用场景对比

| 方式 | 适用场景 | 优点 | 缺点 |
|------|----------|------|------|
| **TranslationManager** | 复杂应用，多个翻译器 | 统一管理、易扩展 | 额外开销 |
| **直接使用翻译器** | 简单应用，单个翻译器 | 配置简单、性能好 | 功能较少 |

## 使用 LanguageIdentifier

使用 `unic-langid` 库提供类型安全的语言定义：

### 基本使用

```rust
use async_translate::{microsoft::{MicrosoftConfig, MicrosoftTranslator}, Translator, LanguageIdentifier};

let translator = MicrosoftTranslator::new(MicrosoftConfig {
    endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
    concurrent_limit: 10,
});

// 解析语言标识符
let chinese: LanguageIdentifier = "zh-CN".parse().unwrap();
let english: LanguageIdentifier = "en".parse().unwrap();

// 使用类型安全的语言标识符
let result = translator.translate_langid("Hello", &chinese).await?;
println!("Translation: {}", result);
```

### 高级功能

```rust
// 指定源语言和目标语言
let result = translator.translate_with_langid("Hello", Some(&english), &chinese).await?;
println!("Translation with source: {}", result);

// 在 Manager 中使用
let result = manager.translate_langid("microsoft", "Hello", &chinese).await?;
println!("Manager translation: {}", result);
```

### LanguageIdentifier 特性

- **类型安全**：编译时检查语言代码有效性
- **IDE 支持**：智能提示和自动补全
- **标准兼容**：符合 BCP 47 和 Unicode 标准
- **错误减少**：避免拼写错误和无效语言代码

### 常用语言标识符

| 语言 | 标识符 |
|------|--------|
| 中文（简体） | `"zh-CN"` |
| 中文（繁体） | `"zh-TW"` |
| 英文 | `"en"` |
| 日文 | `"ja"` |
| 韩文 | `"ko"` |
| 法文 | `"fr"` |
| 德文 | `"de"` |
| 西班牙文 | `"es"` |

### 注意事项

- **并发限制**：两种方式都支持并发限制，但每个翻译器实例独立管理
- **资源管理**：直接使用翻译器需要手动管理实例生命周期
- **扩展性**：需要多个翻译器时建议使用 Manager 方式

## 并发翻译

```rust
let texts = vec!["Text 1", "Text 2", "Text 3"];
let target_lang = "zh";

for text in texts {
    match manager.translate("openai", text, target_lang).await {
        Ok(result) => println!("Translation: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```