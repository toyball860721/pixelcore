# PixelCore Internationalization (i18n) Guide

## Overview

PixelCore provides comprehensive internationalization support for both backend (Rust) and frontend (React) components. The system supports 10 languages out of the box and can be easily extended to support additional languages.

## Supported Languages

1. **English (en)** - Default language
2. **Chinese (zh)** - 中文
3. **Spanish (es)** - Español
4. **French (fr)** - Français
5. **German (de)** - Deutsch
6. **Japanese (ja)** - 日本語
7. **Korean (ko)** - 한국어
8. **Arabic (ar)** - العربية
9. **Portuguese (pt)** - Português
10. **Russian (ru)** - Русский

## Backend (Rust) i18n

### Architecture

The backend i18n system is implemented in the `pixelcore-i18n` crate and provides:

- **Locale Management**: Support for multiple locales with fallback mechanisms
- **Translation System**: Fluent-based translation with built-in translations
- **Date/Time Formatting**: Locale-specific date and time formatting
- **Number Formatting**: Locale-specific number formatting with thousands separators
- **Currency Formatting**: Locale-specific currency formatting with symbols

### Usage Examples

#### 1. Basic Translation

```rust
use pixelcore_i18n::{Translator, SupportedLocale};

#[tokio::main]
async fn main() {
    // Create a translator for Chinese
    let translator = Translator::new(SupportedLocale::Chinese)
        .with_fallback(SupportedLocale::English);

    // Translate a key
    let greeting = translator.translate("greeting").await;
    println!("{}", greeting); // Output: 你好

    // If key doesn't exist, falls back to English or returns the key
    let unknown = translator.translate("unknown.key").await;
}
```

#### 2. Date and Time Formatting

```rust
use pixelcore_i18n::{DateFormatter, SupportedLocale};
use chrono::Utc;

fn main() {
    let formatter = DateFormatter::new(SupportedLocale::Chinese);
    let now = Utc::now();

    // Format date with different styles
    let short = formatter.format_date(&now, "short").unwrap();
    let medium = formatter.format_date(&now, "medium").unwrap();
    let long = formatter.format_date(&now, "long").unwrap();

    // Format date and time together
    let datetime = formatter.format_datetime(&now, "medium").unwrap();

    // Format relative time (e.g., "2 hours ago")
    let relative = formatter.format_relative(&now);
}
```

#### 3. Number Formatting

```rust
use pixelcore_i18n::{NumberFormatter, SupportedLocale};

fn main() {
    let formatter = NumberFormatter::new(SupportedLocale::German);

    // Format number with thousands separators
    let formatted = formatter.format_number(1234567.89, 2);
    println!("{}", formatted); // Output: 1.234.567,89

    // Format percentage
    let percentage = formatter.format_percentage(0.1234, 2);
    println!("{}", percentage); // Output: 12,34%
}
```

#### 4. Currency Formatting

```rust
use pixelcore_i18n::{CurrencyFormatter, SupportedLocale};

fn main() {
    let formatter = CurrencyFormatter::new(SupportedLocale::Spanish);

    // Format currency amount
    let amount = formatter.format_currency(1234.56, "EUR");
    println!("{}", amount); // Output: 1.234,56 €
}
```

### Adding Custom Translations

To add custom translations, modify the `Translator::new()` method in `translator.rs`:

```rust
// Add your custom translations
translations.insert(
    "custom.key".to_string(),
    "Custom translation".to_string()
);
```

## Frontend (React) i18n

### Architecture

The frontend i18n system uses `i18next` and `react-i18next` libraries with:

- **Automatic Language Detection**: Detects user's preferred language from browser settings
- **Local Storage Persistence**: Saves language preference in localStorage
- **Translation Files**: JSON-based translation files for each language
- **React Hooks**: Easy-to-use hooks for accessing translations

### Setup

The i18n system is initialized in `src/i18n/config.ts` and should be imported in your main application file:

```typescript
// In your main.tsx or App.tsx
import './i18n/config';
```

### Usage Examples

#### 1. Using the useTranslation Hook

```typescript
import React from 'react';
import { useTranslation } from 'react-i18next';

export const MyComponent: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div>
      <h1>{t('app.title')}</h1>
      <p>{t('app.description')}</p>
    </div>
  );
};
```

#### 2. Using the Language Switcher

```typescript
import React from 'react';
import { LanguageSwitcher } from './components/LanguageSwitcher';

export const App: React.FC = () => {
  return (
    <div>
      <LanguageSwitcher />
      {/* Your app content */}
    </div>
  );
};
```

#### 3. Programmatically Changing Language

```typescript
import { useTranslation } from 'react-i18next';

export const MyComponent: React.FC = () => {
  const { i18n } = useTranslation();

  const switchToSpanish = () => {
    i18n.changeLanguage('es');
  };

  return (
    <button onClick={switchToSpanish}>
      Switch to Spanish
    </button>
  );
};
```

#### 4. Getting Current Language

```typescript
import { useTranslation } from 'react-i18next';

export const MyComponent: React.FC = () => {
  const { i18n } = useTranslation();

  console.log('Current language:', i18n.language);

  return <div>Current language: {i18n.language}</div>;
};
```

### Adding New Translations

To add new translations to the frontend:

1. Open the appropriate language file in `src/i18n/locales/`
2. Add your new translation keys:

```json
{
  "myFeature": {
    "title": "My Feature Title",
    "description": "My feature description"
  }
}
```

3. Use the translation in your component:

```typescript
const { t } = useTranslation();
<h1>{t('myFeature.title')}</h1>
```

## Translation File Structure

### Frontend Translation Files

Translation files are organized hierarchically:

```json
{
  "app": {
    "title": "Application Title",
    "welcome": "Welcome Message"
  },
  "nav": {
    "home": "Home",
    "dashboard": "Dashboard"
  },
  "common": {
    "loading": "Loading...",
    "error": "Error"
  }
}
```

Access nested translations using dot notation: `t('app.title')`, `t('nav.home')`, etc.

## Best Practices

### 1. Use Semantic Keys

Use descriptive, hierarchical keys that reflect the structure of your application:

```typescript
// Good
t('dashboard.metrics.cpu')
t('config.security.password')

// Bad
t('text1')
t('label2')
```

### 2. Provide Context

When translations might be ambiguous, provide context in the key:

```typescript
// Good
t('button.save')
t('message.save.success')

// Bad
t('save')
```

### 3. Handle Pluralization

For languages with complex pluralization rules, use i18next's pluralization features:

```json
{
  "items": "{{count}} item",
  "items_plural": "{{count}} items"
}
```

```typescript
t('items', { count: 1 }); // "1 item"
t('items', { count: 5 }); // "5 items"
```

### 4. Use Interpolation

Pass dynamic values using interpolation:

```json
{
  "greeting": "Hello, {{name}}!"
}
```

```typescript
t('greeting', { name: 'John' }); // "Hello, John!"
```

### 5. Fallback Strategy

Always provide English translations as a fallback. The system will automatically fall back to English if a translation is missing in the selected language.

## Testing i18n

### Backend Tests

Run the i18n tests:

```bash
cd crates/pixelcore-i18n
cargo test
```

### Frontend Tests

Test language switching:

```typescript
import { renderHook, act } from '@testing-library/react';
import { useTranslation } from 'react-i18next';

test('changes language', () => {
  const { result } = renderHook(() => useTranslation());

  act(() => {
    result.current.i18n.changeLanguage('es');
  });

  expect(result.current.i18n.language).toBe('es');
});
```

## Performance Considerations

### Backend

- Translations are loaded once at initialization
- Formatters are lightweight and can be created on-demand
- Use caching for frequently accessed translations

### Frontend

- Translation files are bundled with the application
- Language changes are instant (no network requests)
- localStorage caching prevents language detection on every page load

## Adding a New Language

### Backend

1. Add the new locale to `SupportedLocale` enum in `locale.rs`:

```rust
pub enum SupportedLocale {
    // ... existing locales
    Italian,
}
```

2. Add locale code mapping in `Locale::from_code()`:

```rust
"it" => Some(SupportedLocale::Italian),
```

3. Add translations in `Translator::new()`:

```rust
SupportedLocale::Italian => {
    translations.insert("greeting".to_string(), "Ciao".to_string());
    // ... more translations
}
```

4. Add date/number/currency formatting patterns in respective formatters.

### Frontend

1. Create a new translation file: `src/i18n/locales/it.json`

2. Add the language to the resources in `config.ts`:

```typescript
import itTranslations from './locales/it.json';

const resources = {
  // ... existing languages
  it: { translation: itTranslations },
};
```

3. Add the language to the LanguageSwitcher:

```typescript
const languages: Language[] = [
  // ... existing languages
  { code: 'it', name: 'Italian', nativeName: 'Italiano' },
];
```

## Troubleshooting

### Translation Not Showing

1. Check that the translation key exists in the JSON file
2. Verify the language code is correct
3. Check browser console for i18next warnings
4. Ensure i18n config is imported before using translations

### Language Not Persisting

1. Check localStorage is enabled in the browser
2. Verify the detection configuration in `config.ts`
3. Clear localStorage and try again

### Formatting Issues

1. Verify the locale is supported
2. Check the format pattern is correct
3. Ensure the input data type matches the formatter expectations

## Resources

- [i18next Documentation](https://www.i18next.com/)
- [react-i18next Documentation](https://react.i18next.com/)
- [Fluent Project](https://projectfluent.org/)
- [Unicode CLDR](http://cldr.unicode.org/)

## Support

For issues or questions about i18n:

1. Check this guide first
2. Review the example code in the codebase
3. Check the test files for usage examples
4. Open an issue on the project repository

---

**Last Updated**: 2024-03-04
**Version**: 1.0.0

