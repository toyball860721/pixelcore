use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported locales
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedLocale {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh")]
    Chinese,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "ar")]
    Arabic,
    #[serde(rename = "pt")]
    Portuguese,
    #[serde(rename = "ru")]
    Russian,
}

impl SupportedLocale {
    /// Get locale code
    pub fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Chinese => "zh",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::German => "de",
            Self::Japanese => "ja",
            Self::Korean => "ko",
            Self::Arabic => "ar",
            Self::Portuguese => "pt",
            Self::Russian => "ru",
        }
    }

    /// Get locale name
    pub fn name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Chinese => "中文",
            Self::Spanish => "Español",
            Self::French => "Français",
            Self::German => "Deutsch",
            Self::Japanese => "日本語",
            Self::Korean => "한국어",
            Self::Arabic => "العربية",
            Self::Portuguese => "Português",
            Self::Russian => "Русский",
        }
    }

    /// Check if locale is RTL (Right-to-Left)
    pub fn is_rtl(&self) -> bool {
        matches!(self, Self::Arabic)
    }

    /// Parse from string
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Self::English),
            "zh" => Some(Self::Chinese),
            "es" => Some(Self::Spanish),
            "fr" => Some(Self::French),
            "de" => Some(Self::German),
            "ja" => Some(Self::Japanese),
            "ko" => Some(Self::Korean),
            "ar" => Some(Self::Arabic),
            "pt" => Some(Self::Portuguese),
            "ru" => Some(Self::Russian),
            _ => None,
        }
    }

    /// Get all supported locales
    pub fn all() -> Vec<Self> {
        vec![
            Self::English,
            Self::Chinese,
            Self::Spanish,
            Self::French,
            Self::German,
            Self::Japanese,
            Self::Korean,
            Self::Arabic,
            Self::Portuguese,
            Self::Russian,
        ]
    }
}

impl fmt::Display for SupportedLocale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl Default for SupportedLocale {
    fn default() -> Self {
        Self::English
    }
}

/// Locale information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Locale {
    pub locale: SupportedLocale,
    pub fallback: Option<SupportedLocale>,
}

impl Locale {
    pub fn new(locale: SupportedLocale) -> Self {
        Self {
            locale,
            fallback: Some(SupportedLocale::English),
        }
    }

    pub fn with_fallback(locale: SupportedLocale, fallback: SupportedLocale) -> Self {
        Self {
            locale,
            fallback: Some(fallback),
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new(SupportedLocale::English)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_code() {
        assert_eq!(SupportedLocale::English.code(), "en");
        assert_eq!(SupportedLocale::Chinese.code(), "zh");
        assert_eq!(SupportedLocale::Arabic.code(), "ar");
    }

    #[test]
    fn test_locale_rtl() {
        assert!(!SupportedLocale::English.is_rtl());
        assert!(SupportedLocale::Arabic.is_rtl());
    }

    #[test]
    fn test_locale_from_code() {
        assert_eq!(
            SupportedLocale::from_code("en"),
            Some(SupportedLocale::English)
        );
        assert_eq!(
            SupportedLocale::from_code("zh"),
            Some(SupportedLocale::Chinese)
        );
        assert_eq!(SupportedLocale::from_code("invalid"), None);
    }

    #[test]
    fn test_all_locales() {
        let locales = SupportedLocale::all();
        assert_eq!(locales.len(), 10);
    }
}
