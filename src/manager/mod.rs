//! 翻译管理器实现

use crate::{error::TranslationError, options::TranslateOptions, translator::Translator};
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

    /// 使用指定的翻译器翻译文本（带配置选项）
    ///
    /// # 参数
    ///
    /// * `translator_name` - 翻译器名称
    /// * `text` - 需要翻译的文本
    /// * `target_lang` - 目标语言标识符
    /// * `source_lang` - 源语言标识符 (None表示自动检测)
    /// * `options` - 翻译配置选项
    ///
    /// # 返回值
    ///
    /// 返回翻译后的文本，如果出错则返回错误信息
    pub async fn translate_with_options(
        &self,
        translator_name: &str,
        text: &str,
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
        options: &TranslateOptions,
    ) -> Result<String, TranslationError> {
        let translator = self.translators.get(translator_name).ok_or_else(|| {
            TranslationError::ConfigurationError(format!(
                "Translator '{}' not found",
                translator_name
            ))
        })?;

        translator
            .translate_with_options(text, target_lang, source_lang, options)
            .await
    }

    /// 使用指定的翻译器翻译文本（使用默认选项）
    ///
    /// # 参数
    ///
    /// * `translator_name` - 翻译器名称
    /// * `text` - 需要翻译的文本
    /// * `target_lang` - 目标语言标识符
    /// * `source_lang` - 源语言标识符 (None表示自动检测)
    ///
    /// # 返回值
    ///
    /// 返回翻译后的文本，如果出错则返回错误信息
    pub async fn translate(
        &self,
        translator_name: &str,
        text: &str,
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
    ) -> Result<String, TranslationError> {
        self.translate_with_options(
            translator_name,
            text,
            target_lang,
            source_lang,
            &TranslateOptions::default(),
        )
        .await
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
