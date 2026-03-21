import { Helmet } from 'react-helmet-async';
import { useTranslation } from 'react-i18next';

/* -------------------------------------------------------------------------- */
/*  Types                                                                     */
/* -------------------------------------------------------------------------- */

interface ArticleMeta {
  publishedTime: string;
  modifiedTime: string;
  author: string;
  tags: string[];
}

interface BreadcrumbItem {
  name: string;
  url: string;
}

interface SEOProps {
  title: string;
  description: string;
  canonical: string;
  ogType?: string;
  ogImage?: string;
  article?: ArticleMeta;
  breadcrumbs?: BreadcrumbItem[];
  jsonLd?: object | object[];
  noindex?: boolean;
}

/* -------------------------------------------------------------------------- */
/*  Constants                                                                 */
/* -------------------------------------------------------------------------- */

const SITE_NAME = 'AigisCode';
const SITE_URL = 'https://aigiscode.com';
const DEFAULT_OG_IMAGE = `${SITE_URL}/og-image.jpg`;
const TWITTER_HANDLE = '@aigiscode';

/** Map i18n language codes to OG locale format */
const LOCALE_MAP: Record<string, string> = {
  en: 'en_US',
  cs: 'cs_CZ',
  fr: 'fr_FR',
  es: 'es_ES',
  zh: 'zh_CN',
  hi: 'hi_IN',
  pt: 'pt_BR',
  pl: 'pl_PL',
  ar: 'ar_SA',
  bn: 'bn_BD',
};

/** All supported languages for hreflang */
const HREFLANG_CODES = ['en', 'cs', 'fr', 'es', 'zh', 'hi', 'pt', 'pl', 'ar', 'bn'] as const;

/* -------------------------------------------------------------------------- */
/*  JSON-LD Helpers                                                           */
/* -------------------------------------------------------------------------- */

function buildBreadcrumbSchema(items: BreadcrumbItem[]): object {
  return {
    '@context': 'https://schema.org',
    '@type': 'BreadcrumbList',
    itemListElement: items.map((item, index) => ({
      '@type': 'ListItem',
      position: index + 1,
      name: item.name,
      item: item.url,
    })),
  };
}

function renderJsonLdScript(data: object): string {
  return JSON.stringify(data);
}

/* -------------------------------------------------------------------------- */
/*  SEO Component                                                             */
/* -------------------------------------------------------------------------- */

export default function SEO({
  title,
  description,
  canonical,
  ogType = 'website',
  ogImage,
  article,
  breadcrumbs,
  jsonLd,
  noindex = false,
}: SEOProps) {
  const { i18n } = useTranslation();
  const currentLang = i18n.language?.slice(0, 2) || 'en';
  const ogLocale = LOCALE_MAP[currentLang] || 'en_US';
  const resolvedImage = ogImage || DEFAULT_OG_IMAGE;
  const robotsContent = noindex
    ? 'noindex, nofollow'
    : 'index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1';

  return (
    <Helmet>
      {/* HTML lang attribute */}
      <html lang={currentLang} />

      {/* Primary */}
      <title>{title}</title>
      <meta name="description" content={description} />
      <meta name="robots" content={robotsContent} />
      <link rel="canonical" href={canonical} />

      {/* Hreflang alternates — path-based: /cs/blog, /fr/features, etc. */}
      {HREFLANG_CODES.map((lang) => {
        const path = canonical.replace(SITE_URL, '');
        const langUrl = lang === 'en'
          ? canonical
          : `${SITE_URL}/${lang}${path || '/'}`;
        return (
          <link key={lang} rel="alternate" hrefLang={lang} href={langUrl} />
        );
      })}
      <link rel="alternate" hrefLang="x-default" href={canonical} />

      {/* Open Graph */}
      <meta property="og:title" content={title} />
      <meta property="og:description" content={description} />
      <meta property="og:url" content={canonical} />
      <meta property="og:type" content={article ? 'article' : ogType} />
      <meta property="og:image" content={resolvedImage} />
      <meta property="og:image:width" content="1200" />
      <meta property="og:image:height" content="630" />
      <meta property="og:site_name" content={SITE_NAME} />
      <meta property="og:locale" content={ogLocale} />
      {HREFLANG_CODES.filter((l) => l !== currentLang).map((lang) => (
        <meta key={lang} property="og:locale:alternate" content={LOCALE_MAP[lang]} />
      ))}

      {/* Article-specific OG tags */}
      {article && (
        <meta property="article:published_time" content={article.publishedTime} />
      )}
      {article && (
        <meta property="article:modified_time" content={article.modifiedTime} />
      )}
      {article && (
        <meta property="article:author" content={article.author} />
      )}
      {article?.tags.map((tag) => (
        <meta key={tag} property="article:tag" content={tag} />
      ))}

      {/* Twitter Card */}
      <meta name="twitter:card" content="summary_large_image" />
      <meta name="twitter:site" content={TWITTER_HANDLE} />
      <meta name="twitter:title" content={title} />
      <meta name="twitter:description" content={description} />
      <meta name="twitter:image" content={resolvedImage} />

      {/* Breadcrumb JSON-LD */}
      {breadcrumbs && breadcrumbs.length > 0 && (
        <script type="application/ld+json">
          {renderJsonLdScript(buildBreadcrumbSchema(breadcrumbs))}
        </script>
      )}

      {/* Custom JSON-LD — supports single object or array of objects */}
      {jsonLd && !Array.isArray(jsonLd) && (
        <script type="application/ld+json">
          {renderJsonLdScript(jsonLd)}
        </script>
      )}
      {jsonLd &&
        Array.isArray(jsonLd) &&
        jsonLd.map((block, i) => (
          <script key={i} type="application/ld+json">
            {renderJsonLdScript(block)}
          </script>
        ))}
    </Helmet>
  );
}
