use anyhow::Result;
use async_translate::{
    manager::TranslationManager,
    microsoft::{MicrosoftConfig, MicrosoftTranslator},
    openai::{OpenAIConfig, OpenAITranslator},
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 创建翻译管理器
    let mut manager = TranslationManager::new();

    // 配置OpenAI翻译器（请替换为实际的API Key）
    let openai_config = OpenAIConfig {
        base_url: "https://api.openai.com/v1".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        api_keys: vec!["your-openai-api-key".to_string()], // 替换为实际的API Key
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

    // 测试翻译
    let text = "Hello, world!";
    let target_lang = "zh";

    info!("Translating: '{}' to {}", text, target_lang);

    // 尝试使用OpenAI翻译
    match manager.translate("openai", text, target_lang).await {
        Ok(result) => info!("OpenAI Translation: {}", result),
        Err(e) => error!("OpenAI Translation error: {}", e),
    }

    // 尝试使用微软翻译
    match manager.translate("microsoft", text, target_lang).await {
        Ok(result) => info!("Microsoft Translation: {}", result),
        Err(e) => error!("Microsoft Translation error: {}", e),
    }

    Ok(())
}
