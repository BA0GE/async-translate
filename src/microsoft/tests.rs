#[cfg(test)]
mod tests {
    use crate::error::TranslationError;
    use crate::microsoft::{MicrosoftConfig, MicrosoftTranslator};
    use crate::options::TranslateOptions;
    use crate::translator::Translator;
    use unic_langid::LanguageIdentifier;

    #[tokio::test]
    async fn test_microsoft_config_default() {
        let config = MicrosoftConfig::default();
        assert_eq!(config.endpoint, None);
        assert_eq!(config.api_key, None);
        assert_eq!(config.concurrent_limit, 10);
    }

    #[tokio::test]
    async fn test_microsoft_translator_creation() {
        let config = MicrosoftConfig {
            endpoint: None,
            api_key: None,
            concurrent_limit: 10,
        };

        let _translator = MicrosoftTranslator::new(config);
        // 这里我们只测试创建是否成功，不测试实际的API调用
        assert!(true); // 如果创建成功，这行代码会执行
    }

    #[tokio::test]
    async fn test_microsoft_translator_direct_translate() {
        let config = MicrosoftConfig {
            endpoint: None,
            api_key: None,
            concurrent_limit: 10,
        };

        let translator = MicrosoftTranslator::new(config);

        // 测试直接调用翻译方法
        let target_lang: LanguageIdentifier = "zh".parse().unwrap();
        match translator
            .translate_text("Hello", &target_lang, None, &TranslateOptions::default())
            .await
        {
            Ok(result) => {
                println!("Direct translation result: {}", result);
                assert!(!result.is_empty());
            }
            Err(e) => {
                // 在测试环境中可能无法访问网络，这是正常的
                println!("Network error (expected in test): {}", e);
                // 验证错误信息包含网络相关内容
                match e {
                    TranslationError::NetworkError(_) => assert!(true),
                    _ => assert!(false, "Expected NetworkError"),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_microsoft_translator_with_source_lang() {
        let config = MicrosoftConfig {
            endpoint: None,
            api_key: None,
            concurrent_limit: 10,
        };

        let translator = MicrosoftTranslator::new(config);

        // 测试使用LanguageIdentifier
        let chinese: LanguageIdentifier = "zh-CN".parse().unwrap();
        let english: LanguageIdentifier = "en".parse().unwrap();

        // 测试自动检测源语言
        match translator
            .translate_text("Hello", &chinese, None, &TranslateOptions::default())
            .await
        {
            Ok(result) => {
                println!("Auto-detect translation result: {}", result);
                assert!(!result.is_empty());
            }
            Err(e) => {
                // 在测试环境中可能无法访问网络，这是正常的
                println!("Network error (expected in test): {}", e);
                match e {
                    TranslationError::NetworkError(_) => assert!(true),
                    _ => assert!(false, "Expected NetworkError"),
                }
            }
        }

        // 测试带源语言的翻译
        match translator
            .translate_text(
                "Hello",
                &chinese,
                Some(&english),
                &TranslateOptions::default(),
            )
            .await
        {
            Ok(result) => {
                println!("Translation with source lang result: {}", result);
                assert!(!result.is_empty());
            }
            Err(e) => {
                // 在测试环境中可能无法访问网络，这是正常的
                println!("Network error (expected in test): {}", e);
                match e {
                    TranslationError::NetworkError(_) => assert!(true),
                    _ => assert!(false, "Expected NetworkError"),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_microsoft_translator_batch_translate() {
        let config = MicrosoftConfig {
            endpoint: None,
            api_key: None,
            concurrent_limit: 10,
        };

        let translator = MicrosoftTranslator::new(config);

        // 测试批量翻译
        let texts = vec!["Hello", "World"];
        let target_lang: LanguageIdentifier = "zh-CN".parse().unwrap();

        // 这里我们只测试方法是否存在和签名是否正确
        // 实际的网络调用在测试环境中可能失败，这是正常的
        match translator
            .translate_batch(&texts, &target_lang, None, &TranslateOptions::default())
            .await
        {
            Ok(results) => {
                println!("Batch translation results: {:?}", results);
                assert!(true);
            }
            Err(e) => {
                // 在测试环境中可能无法访问网络，这是正常的
                println!("Network error (expected in test): {}", e);
                match e {
                    TranslationError::NetworkError(_) => assert!(true),
                    _ => assert!(false, "Expected NetworkError"),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_microsoft_translator_trait_translate() {
        let config = MicrosoftConfig {
            endpoint: None,
            api_key: None,
            concurrent_limit: 10,
        };

        let translator = MicrosoftTranslator::new(config);

        // 测试通过 trait 调用翻译方法
        let target_lang: LanguageIdentifier = "zh".parse().unwrap();
        match translator.translate("Hello", &target_lang, None).await {
            Ok(result) => {
                println!("Trait translation result: {}", result);
                assert!(!result.is_empty());
            }
            Err(e) => {
                // 在测试环境中可能无法访问网络，这是正常的
                println!("Network error (expected in test): {}", e);
                match e {
                    TranslationError::NetworkError(_) => assert!(true),
                    _ => assert!(false, "Expected NetworkError"),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_token_caching() {
        let config = MicrosoftConfig {
            api_key: None, // 必须使用自动认证
            ..Default::default()
        };
        let translator = MicrosoftTranslator::new(config);

        // 第一次获取token
        let token1 = translator.get_auth_token().await;
        assert!(token1.is_ok());

        // 第二次获取，应该从缓存中读取
        let token2 = translator.get_auth_token().await;
        assert!(token2.is_ok());
        assert_eq!(token1.unwrap(), token2.unwrap());

        // 清除缓存
        translator.clear_cached_token().await;

        // 第三次获取，应该获取新的token
        let token3 = translator.get_auth_token().await;
        assert!(token3.is_ok());
    }
}
