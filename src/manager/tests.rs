#[cfg(test)]
mod tests {
    use crate::{manager::TranslationManager, translator::Translator};

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
            async fn translate(&self, text: &str, _target_lang: &str) -> anyhow::Result<String> {
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
            async fn translate(&self, text: &str, _target_lang: &str) -> anyhow::Result<String> {
                Ok(format!("Translated: {}", text))
            }
        }
        
        manager.add_translator("mock", Box::new(MockTranslator));
        
        let result = manager.translate("mock", "test", "zh").await.unwrap();
        assert_eq!(result, "Translated: test");
    }
    
    #[tokio::test]
    async fn test_translation_manager_translate_nonexistent() {
        let manager = TranslationManager::new();
        
        let result = manager.translate("nonexistent", "test", "zh").await;
        assert!(result.is_err());
    }
}