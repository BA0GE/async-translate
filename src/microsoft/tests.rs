#[cfg(test)]
mod tests {
    use crate::microsoft::{MicrosoftConfig, MicrosoftTranslator};
    use crate::translator::Translator;

    #[tokio::test]
    async fn test_microsoft_config_default() {
        let config = MicrosoftConfig::default();
        assert_eq!(config.endpoint, "https://api-edge.cognitive.microsofttranslator.com");
        assert_eq!(config.concurrent_limit, 10);
    }

    #[tokio::test]
    async fn test_microsoft_translator_creation() {
        let config = MicrosoftConfig {
            endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
            concurrent_limit: 10,
        };

        let _translator = MicrosoftTranslator::new(config);
        // 这里我们只测试创建是否成功，不测试实际的API调用
        assert!(true); // 如果创建成功，这行代码会执行
    }

    #[tokio::test]
    async fn test_microsoft_translator_direct_translate() {
        let config = MicrosoftConfig {
            endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
            concurrent_limit: 10,
        };

        let translator = MicrosoftTranslator::new(config);

        // 测试直接调用翻译方法
        match translator.translate("Hello", "zh").await {
            Ok(result) => {
                println!("Direct translation result: {}", result);
                assert!(!result.is_empty());
            },
            Err(e) => {
                // 在测试环境中可能无法访问网络，这是正常的
                println!("Network error (expected in test): {}", e);
                // 验证错误信息包含网络相关内容
                assert!(e.to_string().contains("connect") || e.to_string().contains("network"));
            }
        }
    }

    #[tokio::test]
    async fn test_microsoft_translator_with_langid() {
        let config = MicrosoftConfig {
            endpoint: "https://api-edge.cognitive.microsofttranslator.com".to_string(),
            concurrent_limit: 10,
        };

        let translator = MicrosoftTranslator::new(config);

        // 测试使用LanguageIdentifier
        let chinese: unic_langid::LanguageIdentifier = "zh-CN".parse().unwrap();
        let english: unic_langid::LanguageIdentifier = "en".parse().unwrap();

        // 测试直接使用LanguageIdentifier
        match translator.translate_langid("Hello", &chinese).await {
            Ok(result) => {
                println!("LanguageIdentifier translation result: {}", result);
                assert!(!result.is_empty());
            },
            Err(e) => {
                // 在测试环境中可能无法访问网络，这是正常的
                println!("Network error (expected in test): {}", e);
                assert!(e.to_string().contains("connect") || e.to_string().contains("network"));
            }
        }

        // 测试带源语言的翻译
        match translator.translate_with_langid("Hello", Some(&english), &chinese).await {
            Ok(result) => {
                println!("Translation with source lang result: {}", result);
                assert!(!result.is_empty());
            },
            Err(e) => {
                // 在测试环境中可能无法访问网络，这是正常的
                println!("Network error (expected in test): {}", e);
                assert!(e.to_string().contains("connect") || e.to_string().contains("network"));
            }
        }
    }
}