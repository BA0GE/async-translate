# async-translate

[![Crates.io](https://img.shields.io/crates/v/async-translate.svg)](https://crates.io/crates/async-translate)
[![Documentation](https://docs.rs/async-translate/badge.svg)](https://docs.rs/async-translate)
[![License](https://img.shields.io/crates/l/async-translate.svg)](https://github.com/ba0ge/async-translate/blob/main/LICENSE)

一个支持并发的翻译库，目前支持 OpenAI 和微软翻译服务。

## 功能特性

- **OpenAI 翻译支持**：
  - 支持自定义 base URL、模型、API Key
  - 支持多 API Key 配置，每个 Key 单独计算并发数和 RPM
  - RPM 和并发数可选配置，具备默认数值
  - 自动轮询选择 API Key 以实现负载均衡
  - 自动遵守 RPM 限制（可选）
  - 支持自定义系统提示词

- **微软翻译支持**：
  - 自动获取临时认证token，无需配置API密钥
  - 支持并发操作限制

- **并发支持**：
  - 异步并发翻译
  - 可配置的并发限制
  - 线程安全

- **灵活配置**：
  - 可选择是否启用 RPM 限制
  - 高性能实现，无不必要的开销
  - 支持自定义提示词

- **类型安全**：
  - 支持 LanguageIdentifier，提供编译时类型安全
  - 支持源语言和目标语言同时指定
  - 符合 Unicode BCP 47 标准

- **统一接口**：
  - 统一的翻译接口，易于扩展
  - 灵活的翻译管理器
  - 支持字符串和类型安全两种语言定义方式

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
async-translate = "0.1"
```

## 使用方法

### 基本用法

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
    let openai_config = OpenAIConfig {
        base_url: "https://api.openai.com/v1".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        api_keys: vec!["your-openai-api-key".to_string()],
        rpm_limit: 60,
        concurrent_limit: 10,
        system_prompt: None, // 使用默认提示词
    };
    let openai_translator = Box::new(OpenAITranslator::new(openai_config));
    manager.add_translator("openai", openai_translator);
    
    // 配置微软翻译器（自动获取认证token）
    let microsoft_config = MicrosoftConfig {
        endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
        concurrent_limit: 10,
    };
    let microsoft_translator = Box::new(MicrosoftTranslator::new(microsoft_config));
    manager.add_translator("microsoft", microsoft_translator);
    
    // 执行翻译
    let text = "Hello, world!";
    let target_lang = "zh";
    
    let result = manager.translate("openai", text, target_lang).await?;
    println!("OpenAI Translation: {}", result);
    
    let result = manager.translate("microsoft", text, target_lang).await?;
    println!("Microsoft Translation: {}", result);
    
    Ok(())
}
```

### 使用自定义提示词

```rust
// 配置 OpenAI 翻译器（使用自定义提示词）
let openai_config = OpenAIConfig {
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-3.5-turbo".to_string(),
    api_keys: vec!["your-openai-api-key".to_string()],
    rpm_limit: 60,
    concurrent_limit: 10,
    system_prompt: Some("You are a professional translator with expertise in technical documentation. Please translate the following text to high-quality {target_lang} while preserving technical accuracy.".to_string()),
};
```

### 多 API Key 配置

```rust
let openai_config = OpenAIConfig {
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-3.5-turbo".to_string(),
    api_keys: vec![
        "your-first-openai-api-key".to_string(),
        "your-second-openai-api-key".to_string(),
        "your-third-openai-api-key".to_string(),
    ],
    rpm_limit: 60,           // 每个 Key 每分钟最多 60 个请求
    concurrent_limit: 10,    // 每个 Key 最多 10 个并发请求
    system_prompt: None,    // 使用默认提示词
};
```

### 不限制 RPM 的配置

```rust
let openai_config = OpenAIConfig {
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-3.5-turbo".to_string(),
    api_keys: vec!["your-api-key".to_string()],
    rpm_limit: 0,            // 不限制 RPM
    concurrent_limit: 10,    // 仍然限制并发数
    system_prompt: None,    // 使用默认提示词
};
```

### 直接使用翻译器（无需 Manager）

对于简单场景，你可以直接使用翻译器实例，无需通过 TranslationManager：

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

#### 直接使用 vs Manager 方式

- **直接使用**：适合简单场景，单个翻译器，配置简单
- **Manager 方式**：适合复杂场景，多个翻译器，统一管理

两种方式的并发限制都独立生效，不会相互影响。

### 使用 LanguageIdentifier（类型安全）

使用 `unic-langid` 库提供类型安全的语言定义：

```rust
use anyhow::Result;
use async_translate::{microsoft::{MicrosoftConfig, MicrosoftTranslator}, Translator, LanguageIdentifier};

#[tokio::main]
async fn main() -> Result<()> {
    let translator = MicrosoftTranslator::new(MicrosoftConfig {
        endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
        concurrent_limit: 10,
    });

    // 解析语言标识符
    let chinese: LanguageIdentifier = "zh-CN".parse().unwrap();
    let english: LanguageIdentifier = "en".parse().unwrap();

    // 使用类型安全的语言标识符
    let result = translator.translate_langid("Hello", &chinese).await?;
    println!("Translation: {}", result); // 输出: 你好

    // 指定源语言和目标语言
    let result = translator.translate_with_langid("Hello", Some(&english), &chinese).await?;
    println!("Translation with source: {}", result); // 输出: 你好

    Ok(())
}
```

#### LanguageIdentifier 的优势

- **类型安全**：编译时检查语言代码的有效性
- **IDE 支持**：智能提示和自动补全
- **标准兼容**：符合 BCP 47 和 Unicode 标准
- **错误减少**：避免拼写错误和无效语言代码

### 并发翻译多个文本

```rust
use anyhow::Result;
use std::sync::Arc;
use async_translate::manager::TranslationManager;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建翻译管理器（配置略）
    let manager = Arc::new(manager);
    
    // 要翻译的文本列表
    let texts = vec![
        "Hello, world!",
        "How are you today?",
        "The weather is nice.",
    ];
    
    let target_lang = "zh";
    
    // 并发翻译多个文本
    let mut tasks = vec![];
    
    for text in texts {
        let manager = Arc::clone(&manager);
        let text = text.clone();
        let target_lang = target_lang.clone();
        
        let task = tokio::spawn(async move {
            match manager.translate("openai", &text, &target_lang).await {
                Ok(result) => {
                    println!("Translation: '{}' -> '{}'", text, result);
                    Some(result)
                },
                Err(e) => {
                    eprintln!("Translation error for '{}': {}", text, e);
                    None
                }
            }
        });
        tasks.push(task);
    }
    
    // 等待所有翻译任务完成
    for task in tasks {
        task.await?;
    }
    
    Ok(())
}
```

## 配置说明

### OpenAI 翻译器配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `base_url` | `String` | `"https://api.openai.com/v1"` | OpenAI API 的基础 URL |
| `model` | `String` | `"gpt-3.5-turbo"` | 使用的模型名称 |
| `api_keys` | `Vec<String>` | `vec![]` | API Key 列表，支持多个 Key |
| `rpm_limit` | `u32` | `60` | 每分钟请求数限制，设为0表示不限制 |
| `concurrent_limit` | `usize` | `10` | 并发请求数限制 |
| `system_prompt` | `Option<String>` | `None` | 自定义系统提示词，None表示使用默认提示词 |

### 微软翻译器配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `endpoint` | `String` | `"https://api-edge.cognitive.microsofttranslator.com"` | 微软翻译服务的端点 |
| `concurrent_limit` | `usize` | `10` | 并发请求数限制 |

## 运行示例

```bash
# 完整功能示例（包含 Manager、直接使用和 LanguageIdentifier）
cargo run --example translation_example

# LanguageIdentifier 专门示例
cargo run --example langid_example
```

## 测试

运行单元测试：

```bash
cargo test
```

## 许可证

本项目采用 MIT 许可证。查看 [LICENSE](LICENSE) 文件了解更多信息。