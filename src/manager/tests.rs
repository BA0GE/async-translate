#[cfg(test)]
mod tests {
    use crate::{
        error::TranslationError, manager::TranslationManager, options::TranslateOptions,
        translator::Translator,
    };
    use unic_langid::LanguageIdentifier;

    #[tokio::test]
    async fn test_translation_manager_creation() {
        let manager = TranslationManager::new();
        assert_eq!(manager.list_translators().len(), 0);
    }

    #[tokio::test]
    async fn test_translation_manager_add_translator() {
        let mut manager = TranslationManager::new();

        // 添加一个模拟翻译器
        struct MockTranslator;
        #[async_trait::async_trait]
        impl Translator for MockTranslator {
            async fn translate_with_options(
                &self,
                text: &str,
                _target_lang: &LanguageIdentifier,
                _source_lang: Option<&LanguageIdentifier>,
                _options: &TranslateOptions,
            ) -> Result<String, TranslationError> {
                Ok(format!("Translated: {}", text))
            }
        }

        manager.add_translator("mock", Box::new(MockTranslator));
        assert_eq!(manager.list_translators().len(), 1);
        assert!(manager.has_translator("mock"));
        assert!(!manager.has_translator("nonexistent"));
    }

    #[tokio::test]
    async fn test_translation_manager_translate() {
        let mut manager = TranslationManager::new();

        // 添加一个模拟翻译器
        struct MockTranslator;
        #[async_trait::async_trait]
        impl Translator for MockTranslator {
            async fn translate_with_options(
                &self,
                text: &str,
                _target_lang: &LanguageIdentifier,
                _source_lang: Option<&LanguageIdentifier>,
                _options: &TranslateOptions,
            ) -> Result<String, TranslationError> {
                Ok(format!("Translated: {}", text))
            }
        }

        manager.add_translator("mock", Box::new(MockTranslator));

        let target_lang: LanguageIdentifier = "zh".parse().unwrap();
        let result = manager
            .translate("mock", "test", &target_lang, None)
            .await
            .unwrap();
        assert_eq!(result, "Translated: test");
    }

    #[tokio::test]
    async fn test_translation_manager_translate_nonexistent() {
        let manager = TranslationManager::new();

        let target_lang: LanguageIdentifier = "zh".parse().unwrap();
        let result = manager
            .translate("nonexistent", "test", &target_lang, None)
            .await;
        assert!(result.is_err());
    }
}
