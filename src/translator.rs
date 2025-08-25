//! 翻译器trait定义

use anyhow::Result;
use unic_langid::LanguageIdentifier;

/// 翻译器trait，定义了统一的翻译接口
#[async_trait::async_trait]
pub trait Translator: Send + Sync {
    /// 翻译文本 (使用字符串语言代码)
    ///
    /// # 参数
    ///
    /// * `text` - 需要翻译的文本
    /// * `target_lang` - 目标语言代码（如 "zh" 表示中文，"en" 表示英文）
    ///
    /// # 返回值
    ///
    /// 返回翻译后的文本，如果出错则返回错误信息
    async fn translate(&self, text: &str, target_lang: &str) -> Result<String>;

    /// 翻译文本 (使用 LanguageIdentifier)
    ///
    /// # 参数
    ///
    /// * `text` - 需要翻译的文本
    /// * `target_lang` - 目标语言标识符
    ///
    /// # 返回值
    ///
    /// 返回翻译后的文本，如果出错则返回错误信息
    async fn translate_langid(&self, text: &str, target_lang: &LanguageIdentifier) -> Result<String> {
        self.translate(text, target_lang.to_string().as_str()).await
    }

    /// 翻译文本 (指定源语言和目标语言)
    ///
    /// # 参数
    ///
    /// * `text` - 需要翻译的文本
    /// * `source_lang` - 源语言标识符 (None表示自动检测)
    /// * `target_lang` - 目标语言标识符
    ///
    /// # 返回值
    ///
    /// 返回翻译后的文本，如果出错则返回错误信息
    async fn translate_with_langid(&self, text: &str, _source_lang: Option<&LanguageIdentifier>, target_lang: &LanguageIdentifier) -> Result<String> {
        // 默认实现忽略源语言参数，使用自动检测
        self.translate_langid(text, target_lang).await
    }
}
