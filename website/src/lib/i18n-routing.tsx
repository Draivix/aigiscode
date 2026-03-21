import { forwardRef } from 'react';
import {
  Link as RouterLink,
  NavLink as RouterNavLink,
  useParams,
  useNavigate,
  useLocation,
  type LinkProps,
  type NavigateOptions,
  type NavLinkProps,
} from 'react-router-dom';
import { supportedLanguages, type SupportedLanguage } from '@/i18n';

/* -------------------------------------------------------------------------- */
/*  Constants                                                                  */
/* -------------------------------------------------------------------------- */

/** Default language does not get a URL prefix */
const DEFAULT_LANG: SupportedLanguage = 'en';

/* -------------------------------------------------------------------------- */
/*  Hooks                                                                      */
/* -------------------------------------------------------------------------- */

/**
 * Returns the current language from the URL `:lang` param.
 * Falls back to 'en' if no valid lang prefix is present.
 */
export function useUrlLang(): SupportedLanguage {
  const { lang } = useParams<{ lang?: string }>();
  if (lang && (supportedLanguages as readonly string[]).includes(lang)) {
    return lang as SupportedLanguage;
  }
  return DEFAULT_LANG;
}

/**
 * Prepend the current URL language prefix to a path.
 * English (default) gets no prefix: `/blog`
 * Other languages get prefixed: `/cs/blog`
 */
export function useLocalizedPath() {
  const lang = useUrlLang();

  return (path: string): string => {
    return localizedPath(path, lang);
  };
}

/**
 * Pure function to build a localized path.
 */
export function localizedPath(path: string, lang: SupportedLanguage): string {
  // Strip any existing lang prefix
  const cleaned = stripLangPrefix(path);

  if (lang === DEFAULT_LANG) {
    return cleaned || '/';
  }
  return `/${lang}${cleaned}`;
}

/**
 * Strip a language prefix from a path if present.
 * `/cs/blog` → `/blog`, `/blog` → `/blog`, `/` → `/`
 */
export function stripLangPrefix(path: string): string {
  const match = path.match(/^\/([a-z]{2})(\/.*|$)/);
  if (match && (supportedLanguages as readonly string[]).includes(match[1])) {
    return match[2] || '/';
  }
  return path;
}

/**
 * Extract language from a URL path.
 * `/cs/blog` → 'cs', `/blog` → 'en'
 */
export function langFromPath(path: string): SupportedLanguage {
  const match = path.match(/^\/([a-z]{2})(\/|$)/);
  if (match && (supportedLanguages as readonly string[]).includes(match[1])) {
    return match[1] as SupportedLanguage;
  }
  return DEFAULT_LANG;
}

/* -------------------------------------------------------------------------- */
/*  Language-aware navigation                                                  */
/* -------------------------------------------------------------------------- */

/**
 * Hook that returns a navigate function which preserves the language prefix.
 */
export function useLocalizedNavigate() {
  const navigate = useNavigate();
  const lang = useUrlLang();

  return (to: string, options?: NavigateOptions) => {
    navigate(localizedPath(to, lang), options);
  };
}

/**
 * Switch language by navigating to the same page with a different lang prefix.
 */
export function useSwitchLanguage() {
  const navigate = useNavigate();
  const location = useLocation();

  return (newLang: SupportedLanguage) => {
    const pathWithoutLang = stripLangPrefix(location.pathname);
    const newPath = localizedPath(pathWithoutLang, newLang);
    navigate(newPath + location.search + location.hash, { replace: true });
  };
}

/* -------------------------------------------------------------------------- */
/*  Components                                                                 */
/* -------------------------------------------------------------------------- */

/**
 * Drop-in replacement for react-router-dom's <Link> that automatically
 * prepends the current language prefix to the `to` prop.
 */
export const LocalizedLink = forwardRef<HTMLAnchorElement, LinkProps>(
  function LocalizedLink({ to, ...props }, ref) {
    const lp = useLocalizedPath();

    const localizedTo = typeof to === 'string' ? lp(to) : to;

    return <RouterLink ref={ref} to={localizedTo} {...props} />;
  },
);

/**
 * Drop-in replacement for react-router-dom's <NavLink>.
 */
export const LocalizedNavLink = forwardRef<HTMLAnchorElement, NavLinkProps>(
  function LocalizedNavLink({ to, ...props }, ref) {
    const lp = useLocalizedPath();
    const localizedTo = typeof to === 'string' ? lp(to) : to;
    return <RouterNavLink ref={ref} to={localizedTo} {...props} />;
  },
);

export { DEFAULT_LANG };
