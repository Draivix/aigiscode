import { useEffect } from 'react';
import { Outlet, useParams, useNavigate, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import {
  supportedLanguages,
  type SupportedLanguage,
  changeLanguageSafe,
  resolveLanguage,
} from '@/i18n';

/**
 * Top-level route wrapper that synchronizes the `:lang` URL parameter
 * with i18next. Placed above <Layout> in the route tree.
 *
 * URL patterns:
 *   /blog          → English (default, no prefix)
 *   /cs/blog       → Czech
 *   /fr/blog       → French
 *
 * If a visitor arrives at `/` with a non-English browser language,
 * they are redirected to `/<lang>/` once on the initial visit.
 */
export default function LanguageLayout() {
  const { lang } = useParams<{ lang?: string }>();
  const { i18n } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();

  const isValidLang = lang && (supportedLanguages as readonly string[]).includes(lang);
  const urlLang: SupportedLanguage = isValidLang ? (lang as SupportedLanguage) : 'en';

  // Sync i18next with URL language
  useEffect(() => {
    const currentLang = resolveLanguage(i18n.language) ?? 'en';
    if (currentLang !== urlLang) {
      changeLanguageSafe(urlLang);
    }
  }, [urlLang, i18n]);

  // On first visit to `/` (no lang prefix), detect browser language and redirect
  useEffect(() => {
    if (lang) return; // URL already has a lang prefix, don't redirect

    const detected = resolveLanguage(navigator.language);
    if (detected && detected !== 'en') {
      // Check if user hasn't explicitly chosen English before
      const stored = localStorage.getItem('i18nextLng');
      const storedResolved = resolveLanguage(stored ?? undefined);
      if (!storedResolved || storedResolved === detected) {
        navigate(
          `/${detected}${location.pathname}${location.search}${location.hash}`,
          { replace: true },
        );
      }
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps -- intentionally runs once

  return <Outlet />;
}
