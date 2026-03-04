use crate::error::{I18nError, Result};
use crate::locale::{Locale, SupportedLocale};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Translator for managing translations
pub struct Translator {
    current_locale: Arc<RwLock<Locale>>,
    translations: Arc<RwLock<HashMap<SupportedLocale, HashMap<String, String>>>>,
    locales_path: PathBuf,
}

impl Translator {
    /// Create a new translator
    pub fn new(locales_path: PathBuf) -> Self {
        Self {
            current_locale: Arc::new(RwLock::new(Locale::default())),
            translations: Arc::new(RwLock::new(HashMap::new())),
            locales_path,
        }
    }

    /// Load translations for a locale
    pub async fn load_locale(&self, locale: SupportedLocale) -> Result<()> {
        let locale_path = self.locales_path.join(locale.code());

        if !locale_path.exists() {
            return Err(I18nError::LocaleNotFound(locale.code().to_string()));
        }

        // In a real implementation, this would load .ftl files
        // For now, we'll use a simple key-value approach
        let mut translations = HashMap::new();

        // Load common translations
        translations.insert("app.title".to_string(), self.get_default_translation(locale, "app.title"));
        translations.insert("app.welcome".to_string(), self.get_default_translation(locale, "app.welcome"));
        translations.insert("common.save".to_string(), self.get_default_translation(locale, "common.save"));
        translations.insert("common.cancel".to_string(), self.get_default_translation(locale, "common.cancel"));
        translations.insert("common.delete".to_string(), self.get_default_translation(locale, "common.delete"));
        translations.insert("common.edit".to_string(), self.get_default_translation(locale, "common.edit"));
        translations.insert("common.search".to_string(), self.get_default_translation(locale, "common.search"));
        translations.insert("common.loading".to_string(), self.get_default_translation(locale, "common.loading"));

        let mut trans = self.translations.write().await;
        trans.insert(locale, translations);

        Ok(())
    }

    /// Get default translation (fallback)
    fn get_default_translation(&self, locale: SupportedLocale, key: &str) -> String {
        match (locale, key) {
            (SupportedLocale::English, "app.title") => "PixelCore".to_string(),
            (SupportedLocale::English, "app.welcome") => "Welcome to PixelCore".to_string(),
            (SupportedLocale::English, "common.save") => "Save".to_string(),
            (SupportedLocale::English, "common.cancel") => "Cancel".to_string(),
            (SupportedLocale::English, "common.delete") => "Delete".to_string(),
            (SupportedLocale::English, "common.edit") => "Edit".to_string(),
            (SupportedLocale::English, "common.search") => "Search".to_string(),
            (SupportedLocale::English, "common.loading") => "Loading...".to_string(),

            (SupportedLocale::Chinese, "app.title") => "像素核心".to_string(),
            (SupportedLocale::Chinese, "app.welcome") => "欢迎使用像素核心".to_string(),
            (SupportedLocale::Chinese, "common.save") => "保存".to_string(),
            (SupportedLocale::Chinese, "common.cancel") => "取消".to_string(),
            (SupportedLocale::Chinese, "common.delete") => "删除".to_string(),
            (SupportedLocale::Chinese, "common.edit") => "编辑".to_string(),
            (SupportedLocale::Chinese, "common.search") => "搜索".to_string(),
            (SupportedLocale::Chinese, "common.loading") => "加载中...".to_string(),

            (SupportedLocale::Spanish, "app.title") => "PixelCore".to_string(),
            (SupportedLocale::Spanish, "app.welcome") => "Bienvenido a PixelCore".to_string(),
            (SupportedLocale::Spanish, "common.save") => "Guardar".to_string(),
            (SupportedLocale::Spanish, "common.cancel") => "Cancelar".to_string(),
            (SupportedLocale::Spanish, "common.delete") => "Eliminar".to_string(),
            (SupportedLocale::Spanish, "common.edit") => "Editar".to_string(),
            (SupportedLocale::Spanish, "common.search") => "Buscar".to_string(),
            (SupportedLocale::Spanish, "common.loading") => "Cargando...".to_string(),

            _ => key.to_string(), // Fallback to key
        }
    }

    /// Set current locale
    pub async fn set_locale(&self, locale: Locale) -> Result<()> {
        // Load locale if not already loaded
        {
            let trans = self.translations.read().await;
            if !trans.contains_key(&locale.locale) {
                drop(trans);
                self.load_locale(locale.locale).await?;
            }
        }

        let mut current = self.current_locale.write().await;
        *current = locale;

        Ok(())
    }

    /// Get current locale
    pub async fn current_locale(&self) -> Locale {
        self.current_locale.read().await.clone()
    }

    /// Translate a key
    pub async fn translate(&self, key: &str) -> String {
        let current = self.current_locale.read().await;
        let trans = self.translations.read().await;

        // Try current locale
        if let Some(locale_trans) = trans.get(&current.locale) {
            if let Some(translation) = locale_trans.get(key) {
                return translation.clone();
            }
        }

        // Try fallback locale
        if let Some(fallback) = current.fallback {
            if let Some(locale_trans) = trans.get(&fallback) {
                if let Some(translation) = locale_trans.get(key) {
                    return translation.clone();
                }
            }
        }

        // Return key as fallback
        key.to_string()
    }

    /// Translate with parameters
    pub async fn translate_with_params(
        &self,
        key: &str,
        params: HashMap<String, String>,
    ) -> String {
        let mut translation = self.translate(key).await;

        for (param_key, param_value) in params {
            let placeholder = format!("{{{}}}", param_key);
            translation = translation.replace(&placeholder, &param_value);
        }

        translation
    }
}

/// Translator builder
pub struct TranslatorBuilder {
    locales_path: Option<PathBuf>,
    default_locale: Option<SupportedLocale>,
}

impl TranslatorBuilder {
    pub fn new() -> Self {
        Self {
            locales_path: None,
            default_locale: None,
        }
    }

    pub fn locales_path(mut self, path: PathBuf) -> Self {
        self.locales_path = Some(path);
        self
    }

    pub fn default_locale(mut self, locale: SupportedLocale) -> Self {
        self.default_locale = Some(locale);
        self
    }

    pub async fn build(self) -> Result<Translator> {
        let locales_path = self
            .locales_path
            .unwrap_or_else(|| PathBuf::from("./locales"));

        let translator = Translator::new(locales_path);

        let default_locale = self.default_locale.unwrap_or(SupportedLocale::English);
        translator.load_locale(default_locale).await?;
        translator.set_locale(Locale::new(default_locale)).await?;

        Ok(translator)
    }
}

impl Default for TranslatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_translator_creation() {
        let temp_dir = TempDir::new().unwrap();
        let translator = Translator::new(temp_dir.path().to_path_buf());

        let locale = translator.current_locale().await;
        assert_eq!(locale.locale, SupportedLocale::English);
    }

    #[tokio::test]
    async fn test_translation() {
        let temp_dir = TempDir::new().unwrap();

        // Create locale directories
        std::fs::create_dir_all(temp_dir.path().join("en")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("zh")).unwrap();

        let translator = Translator::new(temp_dir.path().to_path_buf());
        translator.load_locale(SupportedLocale::English).await.unwrap();

        let translation = translator.translate("app.title").await;
        assert_eq!(translation, "PixelCore");
    }

    #[tokio::test]
    async fn test_locale_switching() {
        let temp_dir = TempDir::new().unwrap();

        std::fs::create_dir_all(temp_dir.path().join("en")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("zh")).unwrap();

        let translator = Translator::new(temp_dir.path().to_path_buf());
        translator.load_locale(SupportedLocale::English).await.unwrap();
        translator.load_locale(SupportedLocale::Chinese).await.unwrap();

        // English
        translator.set_locale(Locale::new(SupportedLocale::English)).await.unwrap();
        let translation = translator.translate("common.save").await;
        assert_eq!(translation, "Save");

        // Chinese
        translator.set_locale(Locale::new(SupportedLocale::Chinese)).await.unwrap();
        let translation = translator.translate("common.save").await;
        assert_eq!(translation, "保存");
    }
}
