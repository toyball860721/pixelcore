//! PixelCore Internationalization Module
//!
//! Provides comprehensive i18n support with Fluent localization.

pub mod translator;
pub mod locale;
pub mod formatter;
pub mod error;

pub use translator::{Translator, TranslatorBuilder};
pub use locale::{Locale, SupportedLocale};
pub use formatter::{DateFormatter, NumberFormatter, CurrencyFormatter};
pub use error::{I18nError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that all public exports are accessible
        let _locale: Option<Locale> = None;
    }
}
