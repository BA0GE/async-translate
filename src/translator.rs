//! 翻译器trait定义

use crate::{error::TranslationError, options::TranslateOptions};
use unic_langid::LanguageIdentifier;

/// 翻译器trait，定义了统一的翻译接口
#[async_trait::async_trait]
pub trait Translator: Send + Sync {
    /// 翻译文本（带配置选项）
    ///
    /// # 参数
    ///
    /// * `text` - 需要翻译的文本
    /// * `target_lang` - 目标语言标识符
    /// * `source_lang` - 源语言标识符 (None表示自动检测)
    /// * `options` - 翻译配置选项
    ///
    /// # 返回值
    ///
    /// 返回翻译后的文本，如果出错则返回错误信息
    async fn translate_with_options(
        &self,
        text: &str,
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
        options: &TranslateOptions,
    ) -> Result<String, TranslationError>;

    /// 翻译文本（使用默认选项）
    ///
    /// # 参数
    ///
    /// * `text` - 需要翻译的文本
    /// * `target_lang` - 目标语言标识符
    /// * `source_lang` - 源语言标识符 (None表示自动检测)
    ///
    /// # 返回值
    ///
    /// 返回翻译后的文本，如果出错则返回错误信息
    async fn translate(
        &self,
        text: &str,
        target_lang: &LanguageIdentifier,
        source_lang: Option<&LanguageIdentifier>,
    ) -> Result<String, TranslationError> {
        self.translate_with_options(text, target_lang, source_lang, &TranslateOptions::default())
            .await
    }
}
