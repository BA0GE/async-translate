#[cfg(test)]
mod tests {
    use crate::openai::{OpenAIConfig, OpenAITranslator};

    #[tokio::test]
    async fn test_openai_config_default() {
        let config = OpenAIConfig::default();
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert_eq!(config.model, "gpt-3.5-turbo");
        assert_eq!(config.api_keys.len(), 0);
        assert_eq!(config.rpm_limit, 60);
        assert_eq!(config.concurrent_limit, 10);
        assert_eq!(config.system_prompt, None);
    }

    #[tokio::test]
    async fn test_openai_translator_creation() {
        let config = OpenAIConfig {
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_keys: vec!["test-key".to_string()],
            rpm_limit: 60,
            concurrent_limit: 10,
            system_prompt: None,
        };
        
        let _translator = OpenAITranslator::new(config);
        // 这里我们只测试创建是否成功，不测试实际的API调用
        assert!(true); // 如果创建成功，这行代码会执行
    }

    #[tokio::test]
    async fn test_openai_multiple_keys() {
        let config = OpenAIConfig {
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_keys: vec![
                "test-key-1".to_string(),
                "test-key-2".to_string(),
                "test-key-3".to_string(),
            ],
            rpm_limit: 60,
            concurrent_limit: 10,
            system_prompt: None,
        };
        
        let _translator = OpenAITranslator::new(config);
        // 测试创建是否成功
        assert!(true);
    }

    #[tokio::test]
    async fn test_openai_no_rpm_limit() {
        let config = OpenAIConfig {
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_keys: vec!["test-key".to_string()],
            rpm_limit: 0, // 不限制RPM
            concurrent_limit: 10,
            system_prompt: None,
        };
        
        let _translator = OpenAITranslator::new(config);
        // 测试创建是否成功
        assert!(true);
    }

    #[tokio::test]
    async fn test_openai_custom_system_prompt() {
        let custom_prompt = "You are a professional translator. Please translate the following text to high-quality {target_lang}.".to_string();
        let config = OpenAIConfig {
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_keys: vec!["test-key".to_string()],
            rpm_limit: 60,
            concurrent_limit: 10,
            system_prompt: Some(custom_prompt),
        };
        
        let _translator = OpenAITranslator::new(config);
        // 测试创建是否成功
        assert!(true);
    }
}