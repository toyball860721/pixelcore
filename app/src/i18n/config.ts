import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Import translation files
import enTranslations from './locales/en.json';
import zhTranslations from './locales/zh.json';
import esTranslations from './locales/es.json';
import frTranslations from './locales/fr.json';
import deTranslations from './locales/de.json';
import jaTranslations from './locales/ja.json';
import koTranslations from './locales/ko.json';
import arTranslations from './locales/ar.json';
import ptTranslations from './locales/pt.json';
import ruTranslations from './locales/ru.json';

const resources = {
  en: { translation: enTranslations },
  zh: { translation: zhTranslations },
  es: { translation: esTranslations },
  fr: { translation: frTranslations },
  de: { translation: deTranslations },
  ja: { translation: jaTranslations },
  ko: { translation: koTranslations },
  ar: { translation: arTranslations },
  pt: { translation: ptTranslations },
  ru: { translation: ruTranslations },
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: 'en',
    debug: false,
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
    },
  });

export default i18n;
