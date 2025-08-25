//! 翻译管理器实现

use crate::translator::Translator;
use anyhow::Result;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

/// 翻译管理器，用于统一管理多个翻译器
pub struct TranslationManager {
    /// 翻译器映射表，键为翻译器名称，值为翻译器实例
    translators: HashMap<String, Box<dyn Translator>>,
}

impl TranslationManager {
    /// 创建新的翻译管理器实例
    /// 
    /// # 返回值
    /// 
    /// 返回翻译管理器实例
    pub fn new() -> Self {
        Self {
            translators: HashMap::new(),
        }
    }
    
    /// 添加翻译器到管理器
    /// 
    /// # 参数
    /// 
    /// * `name` - 翻译器名称
    /// * `translator` - 翻译器实例
    pub fn add_translator(&mut self, name: &str, translator: Box<dyn Translator>) {
        self.translators.insert(name.to_string(), translator);
    }
    
    /// 使用指定的翻译器翻译文本
    ///
    /// # 参数
    ///
    /// * `translator_name` - 翻译器名称
    /// * `text` - 需要翻译的文本
    /// * `target_lang` - 目标语言代码
    ///
    /// # 返回值
    ///
    /// 返回翻译后的文本，如果出错则返回错误信息
    pub async fn translate(&self, translator_name: &str, text: &str, target_lang: &str) -> Result<String> {
        let translator = self.translators
            .get(translator_name)
            .ok_or_else(|| anyhow::anyhow!("Translator '{}' not found", translator_name))?;

        translator.translate(text, target_lang).await
    }

    /// 使用指定的翻译器翻译文本 (使用 LanguageIdentifier)
    ///
    /// # 参数
    ///
    /// * `translator_name` - 翻译器名称
    /// * `text` - 需要翻译的文本
    /// * `target_lang` - 目标语言标识符
    ///
    /// # 返回值
    ///
    /// 返回翻译后的文本，如果出错则返回错误信息
    pub async fn translate_langid(&self, translator_name: &str, text: &str, target_lang: &LanguageIdentifier) -> Result<String> {
        let translator = self.translators
            .get(translator_name)
            .ok_or_else(|| anyhow::anyhow!("Translator '{}' not found", translator_name))?;

        translator.translate_langid(text, target_lang).await
    }

    /// 使用指定的翻译器翻译文本 (指定源语言和目标语言)
    ///
    /// # 参数
    ///
    /// * `translator_name` - 翻译器名称
    /// * `text` - 需要翻译的文本
    /// * `source_lang` - 源语言标识符 (None表示自动检测)
    /// * `target_lang` - 目标语言标识符
    ///
    /// # 返回值
    ///
    /// 返回翻译后的文本，如果出错则返回错误信息
    pub async fn translate_with_langid(&self, translator_name: &str, text: &str, source_lang: Option<&LanguageIdentifier>, target_lang: &LanguageIdentifier) -> Result<String> {
        let translator = self.translators
            .get(translator_name)
            .ok_or_else(|| anyhow::anyhow!("Translator '{}' not found", translator_name))?;

        translator.translate_with_langid(text, source_lang, target_lang).await
    }
    
    /// 检查指定的翻译器是否存在
    /// 
    /// # 参数
    /// 
    /// * `translator_name` - 翻译器名称
    /// 
    /// # 返回值
    /// 
    /// 如果翻译器存在返回true，否则返回false
    pub fn has_translator(&self, translator_name: &str) -> bool {
        self.translators.contains_key(translator_name)
    }
    
    /// 获取所有翻译器名称
    /// 
    /// # 返回值
    /// 
    /// 返回所有翻译器名称的向量
    pub fn list_translators(&self) -> Vec<String> {
        self.translators.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests;