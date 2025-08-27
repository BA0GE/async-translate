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
  - **改进的提示工程**：通过系统提示词和示例，确保只返回翻译结果，无额外内容。

- **微软翻译支持**：
  - **改进的认证管理**：自动获取临时认证token，并支持缓存和过期自动刷新。
  - 支持并发操作限制
  - **简化批量翻译结果**：提供 `translate_batch_to_strings` 方法直接返回 `Vec<String>`。

- **并发支持**：
  - 异步并发翻译
  - 可配置的并发限制
  - 线程安全

- **灵活配置**：
  - **新增 Builder 模式**：为 `OpenAIConfig` 和 `MicrosoftConfig` 提供更符合人体工程学的配置方式。
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

- **可配置的超时和重试**：
  - 支持自定义超时时间
  - 支持重试机制
  - 详细的错误信息

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
async-translate = "0.1"
```

## 使用方法

### 基本用法

请参考 `src/main.rs` 中的完整示例，它包含了 Microsoft 和 OpenAI 翻译器的详细用法。

```rust
// 示例代码片段，完整示例请查看 src/main.rs
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

// 详细的 run_microsoft_translator_demo 和 run_openai_translator_demo 函数定义在 src/main.rs 中
```

### 配置示例 (Builder 模式)

#### OpenAI 配置

```rust
use async_translate::openai::OpenAIConfig;

let config = OpenAIConfig::builder()
    .api_keys(vec!["your-openai-api-key"]) // 支持 Vec<&str> 或 Vec<String>
    .base_url("https://api.openai.com/v1")
    .model("gpt-3.5-turbo")
    .rpm_limit(60) // 每分钟请求数限制
    .concurrent_limit(10) // 并发请求数限制
    .system_prompt("You are a helpful assistant.") // 自定义系统提示词
    .build();
```

#### 微软配置

```rust
use async_translate::microsoft::MicrosoftConfig;

let config = MicrosoftConfig::builder()
    .api_key(Some("your-microsoft-api-key")) // 可选，None表示自动认证
    .endpoint("https://api-edge.cognitive.microsofttranslator.com")
    .concurrent_limit(10)
    .build();
```

## 运行示例

```bash
cargo run
```

## 测试

运行单元测试：

```bash
cargo test
```

## 许可证

本项目采用 MIT 许可证。查看 [LICENSE](LICENSE) 文件了解更多信息。
