use crate::{error::I18nError, locale::SupportedLocale};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Date/time formatter with locale-specific patterns
pub struct DateFormatter {
    locale: SupportedLocale,
    patterns: HashMap<String, String>,
}

impl DateFormatter {
    pub fn new(locale: SupportedLocale) -> Self {
        let mut patterns = HashMap::new();

        // Date patterns for different locales
        match locale {
            SupportedLocale::English => {
                patterns.insert("short".to_string(), "%m/%d/%Y".to_string());
                patterns.insert("medium".to_string(), "%b %d, %Y".to_string());
                patterns.insert("long".to_string(), "%B %d, %Y".to_string());
                patterns.insert("full".to_string(), "%A, %B %d, %Y".to_string());
                patterns.insert("time".to_string(), "%I:%M %p".to_string());
            }
            SupportedLocale::Chinese => {
                patterns.insert("short".to_string(), "%Y/%m/%d".to_string());
                patterns.insert("medium".to_string(), "%Y年%m月%d日".to_string());
                patterns.insert("long".to_string(), "%Y年%m月%d日".to_string());
                patterns.insert("full".to_string(), "%Y年%m月%d日 %A".to_string());
                patterns.insert("time".to_string(), "%H:%M".to_string());
            }
            SupportedLocale::Spanish => {
                patterns.insert("short".to_string(), "%d/%m/%Y".to_string());
                patterns.insert("medium".to_string(), "%d de %b de %Y".to_string());
                patterns.insert("long".to_string(), "%d de %B de %Y".to_string());
                patterns.insert("full".to_string(), "%A, %d de %B de %Y".to_string());
                patterns.insert("time".to_string(), "%H:%M".to_string());
            }
            SupportedLocale::French => {
                patterns.insert("short".to_string(), "%d/%m/%Y".to_string());
                patterns.insert("medium".to_string(), "%d %b %Y".to_string());
                patterns.insert("long".to_string(), "%d %B %Y".to_string());
                patterns.insert("full".to_string(), "%A %d %B %Y".to_string());
                patterns.insert("time".to_string(), "%H:%M".to_string());
            }
            SupportedLocale::German => {
                patterns.insert("short".to_string(), "%d.%m.%Y".to_string());
                patterns.insert("medium".to_string(), "%d. %b %Y".to_string());
                patterns.insert("long".to_string(), "%d. %B %Y".to_string());
                patterns.insert("full".to_string(), "%A, %d. %B %Y".to_string());
                patterns.insert("time".to_string(), "%H:%M".to_string());
            }
            SupportedLocale::Japanese => {
                patterns.insert("short".to_string(), "%Y/%m/%d".to_string());
                patterns.insert("medium".to_string(), "%Y年%m月%d日".to_string());
                patterns.insert("long".to_string(), "%Y年%m月%d日".to_string());
                patterns.insert("full".to_string(), "%Y年%m月%d日(%A)".to_string());
                patterns.insert("time".to_string(), "%H:%M".to_string());
            }
            SupportedLocale::Korean => {
                patterns.insert("short".to_string(), "%Y. %m. %d.".to_string());
                patterns.insert("medium".to_string(), "%Y년 %m월 %d일".to_string());
                patterns.insert("long".to_string(), "%Y년 %m월 %d일".to_string());
                patterns.insert("full".to_string(), "%Y년 %m월 %d일 %A".to_string());
                patterns.insert("time".to_string(), "%H:%M".to_string());
            }
            SupportedLocale::Arabic => {
                patterns.insert("short".to_string(), "%d/%m/%Y".to_string());
                patterns.insert("medium".to_string(), "%d %b، %Y".to_string());
                patterns.insert("long".to_string(), "%d %B، %Y".to_string());
                patterns.insert("full".to_string(), "%A، %d %B، %Y".to_string());
                patterns.insert("time".to_string(), "%H:%M".to_string());
            }
            SupportedLocale::Portuguese => {
                patterns.insert("short".to_string(), "%d/%m/%Y".to_string());
                patterns.insert("medium".to_string(), "%d de %b de %Y".to_string());
                patterns.insert("long".to_string(), "%d de %B de %Y".to_string());
                patterns.insert("full".to_string(), "%A, %d de %B de %Y".to_string());
                patterns.insert("time".to_string(), "%H:%M".to_string());
            }
            SupportedLocale::Russian => {
                patterns.insert("short".to_string(), "%d.%m.%Y".to_string());
                patterns.insert("medium".to_string(), "%d %b %Y г.".to_string());
                patterns.insert("long".to_string(), "%d %B %Y г.".to_string());
                patterns.insert("full".to_string(), "%A, %d %B %Y г.".to_string());
                patterns.insert("time".to_string(), "%H:%M".to_string());
            }
        }

        Self { locale, patterns }
    }

    /// Format a date with the specified style
    pub fn format_date(&self, date: &DateTime<Utc>, style: &str) -> Result<String, I18nError> {
        let pattern = self
            .patterns
            .get(style)
            .ok_or_else(|| I18nError::FormatError(format!("Unknown date style: {}", style)))?;

        Ok(date.format(pattern).to_string())
    }

    /// Format a date and time
    pub fn format_datetime(&self, date: &DateTime<Utc>, date_style: &str) -> Result<String, I18nError> {
        let date_pattern = self
            .patterns
            .get(date_style)
            .ok_or_else(|| I18nError::FormatError(format!("Unknown date style: {}", date_style)))?;

        let time_pattern = self
            .patterns
            .get("time")
            .ok_or_else(|| I18nError::FormatError("Time pattern not found".to_string()))?;

        let formatted = format!(
            "{} {}",
            date.format(date_pattern),
            date.format(time_pattern)
        );

        Ok(formatted)
    }

    /// Format relative time (e.g., "2 hours ago")
    pub fn format_relative(&self, date: &DateTime<Utc>) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(*date);

        let seconds = duration.num_seconds();
        let minutes = duration.num_minutes();
        let hours = duration.num_hours();
        let days = duration.num_days();

        match self.locale {
            SupportedLocale::English => {
                if seconds < 60 {
                    "just now".to_string()
                } else if minutes < 60 {
                    format!("{} minute{} ago", minutes, if minutes == 1 { "" } else { "s" })
                } else if hours < 24 {
                    format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
                } else {
                    format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
                }
            }
            SupportedLocale::Chinese => {
                if seconds < 60 {
                    "刚刚".to_string()
                } else if minutes < 60 {
                    format!("{}分钟前", minutes)
                } else if hours < 24 {
                    format!("{}小时前", hours)
                } else {
                    format!("{}天前", days)
                }
            }
            SupportedLocale::Spanish => {
                if seconds < 60 {
                    "ahora mismo".to_string()
                } else if minutes < 60 {
                    format!("hace {} minuto{}", minutes, if minutes == 1 { "" } else { "s" })
                } else if hours < 24 {
                    format!("hace {} hora{}", hours, if hours == 1 { "" } else { "s" })
                } else {
                    format!("hace {} día{}", days, if days == 1 { "" } else { "s" })
                }
            }
            _ => {
                // Fallback to English for other locales
                if seconds < 60 {
                    "just now".to_string()
                } else if minutes < 60 {
                    format!("{} minutes ago", minutes)
                } else if hours < 24 {
                    format!("{} hours ago", hours)
                } else {
                    format!("{} days ago", days)
                }
            }
        }
    }
}

/// Number formatter with locale-specific patterns
pub struct NumberFormatter {
    locale: SupportedLocale,
}

impl NumberFormatter {
    pub fn new(locale: SupportedLocale) -> Self {
        Self { locale }
    }

    /// Format a number with thousands separators
    pub fn format_number(&self, number: f64, decimals: usize) -> String {
        let (thousands_sep, decimal_sep) = self.get_separators();

        let formatted = format!("{:.prec$}", number, prec = decimals);
        let parts: Vec<&str> = formatted.split('.').collect();

        let integer_part = parts[0];
        let decimal_part = if parts.len() > 1 { parts[1] } else { "" };

        // Add thousands separators
        let mut result = String::new();
        let chars: Vec<char> = integer_part.chars().collect();
        let len = chars.len();

        for (i, ch) in chars.iter().enumerate() {
            result.push(*ch);
            let pos = len - i - 1;
            if pos > 0 && pos % 3 == 0 {
                result.push(thousands_sep);
            }
        }

        if !decimal_part.is_empty() {
            result.push(decimal_sep);
            result.push_str(decimal_part);
        }

        result
    }

    /// Format a percentage
    pub fn format_percentage(&self, value: f64, decimals: usize) -> String {
        let formatted = self.format_number(value * 100.0, decimals);
        match self.locale {
            SupportedLocale::French => format!("{} %", formatted),
            _ => format!("{}%", formatted),
        }
    }

    /// Get thousands and decimal separators for the locale
    fn get_separators(&self) -> (char, char) {
        match self.locale {
            SupportedLocale::English => (',', '.'),
            SupportedLocale::Chinese => (',', '.'),
            SupportedLocale::Spanish => ('.', ','),
            SupportedLocale::French => (' ', ','),
            SupportedLocale::German => ('.', ','),
            SupportedLocale::Japanese => (',', '.'),
            SupportedLocale::Korean => (',', '.'),
            SupportedLocale::Arabic => (',', '.'),
            SupportedLocale::Portuguese => ('.', ','),
            SupportedLocale::Russian => (' ', ','),
        }
    }
}

/// Currency formatter with locale-specific symbols and patterns
pub struct CurrencyFormatter {
    locale: SupportedLocale,
}

impl CurrencyFormatter {
    pub fn new(locale: SupportedLocale) -> Self {
        Self { locale }
    }

    /// Format a currency amount
    pub fn format_currency(&self, amount: f64, currency_code: &str) -> String {
        let number_formatter = NumberFormatter::new(self.locale);
        let formatted_amount = number_formatter.format_number(amount, 2);

        let symbol = self.get_currency_symbol(currency_code);

        match self.locale {
            SupportedLocale::English => format!("{}{}", symbol, formatted_amount),
            SupportedLocale::Chinese => format!("{}{}", symbol, formatted_amount),
            SupportedLocale::Spanish => format!("{} {}", formatted_amount, symbol),
            SupportedLocale::French => format!("{} {}", formatted_amount, symbol),
            SupportedLocale::German => format!("{} {}", formatted_amount, symbol),
            SupportedLocale::Japanese => format!("{}{}", symbol, formatted_amount),
            SupportedLocale::Korean => format!("{}{}", symbol, formatted_amount),
            SupportedLocale::Arabic => format!("{} {}", formatted_amount, symbol),
            SupportedLocale::Portuguese => format!("{} {}", symbol, formatted_amount),
            SupportedLocale::Russian => format!("{} {}", formatted_amount, symbol),
        }
    }

    /// Get currency symbol for a currency code
    fn get_currency_symbol(&self, currency_code: &str) -> String {
        let code = currency_code.to_uppercase();
        match code.as_str() {
            "USD" => "$".to_string(),
            "EUR" => "€".to_string(),
            "GBP" => "£".to_string(),
            "JPY" => "¥".to_string(),
            "CNY" => "¥".to_string(),
            "KRW" => "₩".to_string(),
            "RUB" => "₽".to_string(),
            "BRL" => "R$".to_string(),
            _ => currency_code.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_date_formatting() {
        let formatter = DateFormatter::new(SupportedLocale::English);
        let date = Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap();

        let short = formatter.format_date(&date, "short").unwrap();
        assert!(short.contains("03") && short.contains("15") && short.contains("2024"));

        let medium = formatter.format_date(&date, "medium").unwrap();
        assert!(medium.contains("Mar") && medium.contains("15") && medium.contains("2024"));
    }

    #[test]
    fn test_chinese_date_formatting() {
        let formatter = DateFormatter::new(SupportedLocale::Chinese);
        let date = Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap();

        let medium = formatter.format_date(&date, "medium").unwrap();
        assert!(medium.contains("2024年") && medium.contains("03月") && medium.contains("15日"));
    }

    #[test]
    fn test_number_formatting() {
        let formatter = NumberFormatter::new(SupportedLocale::English);
        let formatted = formatter.format_number(1234567.89, 2);
        assert_eq!(formatted, "1,234,567.89");

        let formatter_de = NumberFormatter::new(SupportedLocale::German);
        let formatted_de = formatter_de.format_number(1234567.89, 2);
        assert_eq!(formatted_de, "1.234.567,89");
    }

    #[test]
    fn test_percentage_formatting() {
        let formatter = NumberFormatter::new(SupportedLocale::English);
        let formatted = formatter.format_percentage(0.1234, 2);
        assert_eq!(formatted, "12.34%");
    }

    #[test]
    fn test_currency_formatting() {
        let formatter = CurrencyFormatter::new(SupportedLocale::English);
        let formatted = formatter.format_currency(1234.56, "USD");
        assert_eq!(formatted, "$1,234.56");

        let formatter_es = CurrencyFormatter::new(SupportedLocale::Spanish);
        let formatted_es = formatter_es.format_currency(1234.56, "EUR");
        assert_eq!(formatted_es, "1.234,56 €");
    }

    #[test]
    fn test_relative_time() {
        let formatter = DateFormatter::new(SupportedLocale::English);
        let now = Utc::now();
        let two_hours_ago = now - chrono::Duration::hours(2);

        let relative = formatter.format_relative(&two_hours_ago);
        assert!(relative.contains("2 hours ago"));

        let formatter_zh = DateFormatter::new(SupportedLocale::Chinese);
        let relative_zh = formatter_zh.format_relative(&two_hours_ago);
        assert!(relative_zh.contains("2小时前"));
    }
}
