import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

import en from './locales/en.json';

type LocaleMessages = {
  [key: string]: string | LocaleMessages;
};

/* Supported language codes */
export const supportedLanguages = ['en', 'cs', 'fr', 'es', 'zh', 'hi', 'pt', 'pl', 'ar', 'bn'] as const;
export type SupportedLanguage = (typeof supportedLanguages)[number];

/* Native display names for each language */
export const languageNames: Record<SupportedLanguage, string> = {
  en: 'English',
  cs: 'Čeština',
  fr: 'Français',
  es: 'Español',
  zh: '中文',
  hi: 'हिन्दी',
  pt: 'Português',
  pl: 'Polski',
  ar: 'العربية',
  bn: 'বাংলা',
};

/* Lazy-load non-English locales to keep the main bundle small */
const localeImports: Record<string, () => Promise<{ default: LocaleMessages }>> = {
  cs: () => import('./locales/cs.json'),
  fr: () => import('./locales/fr.json'),
  es: () => import('./locales/es.json'),
  zh: () => import('./locales/zh.json'),
  hi: () => import('./locales/hi.json'),
  pt: () => import('./locales/pt.json'),
  pl: () => import('./locales/pl.json'),
  ar: () => import('./locales/ar.json'),
  bn: () => import('./locales/bn.json'),
};

/**
 * Normalize a language tag like "en-US" or "cs-CZ" to a supported
 * two-letter code. Returns undefined if no match.
 */
export function resolveLanguage(tag: string | undefined): SupportedLanguage | undefined {
  if (!tag) return undefined;
  const short = tag.slice(0, 2).toLowerCase();
  return (supportedLanguages as readonly string[]).includes(short)
    ? (short as SupportedLanguage)
    : undefined;
}

/**
 * Load a locale bundle into i18next if not already loaded.
 * Returns immediately for English (bundled inline).
 */
export async function loadLocale(lang: string): Promise<void> {
  const resolved = resolveLanguage(lang);
  if (!resolved || resolved === 'en') return;
  if (i18n.hasResourceBundle(resolved, 'translation')) return;
  if (!localeImports[resolved]) return;
  const mod = await localeImports[resolved]();
  i18n.addResourceBundle(resolved, 'translation', mod.default, true, true);
}

/**
 * Pre-load the locale bundle, then switch language.
 * This prevents the flash of English text that occurs when
 * i18n.changeLanguage() fires before the async bundle arrives.
 */
export async function changeLanguageSafe(lang: SupportedLanguage): Promise<void> {
  await loadLocale(lang);
  await i18n.changeLanguage(lang);
}

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { translation: en },
    },
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['path', 'localStorage', 'navigator'],
      lookupFromPathIndex: 0,
      caches: ['localStorage'],
    },
  });

/* Eagerly load the detected/persisted language on startup */
const detected = resolveLanguage(i18n.language);
if (detected && detected !== 'en') {
  loadLocale(detected);
}

export default i18n;
