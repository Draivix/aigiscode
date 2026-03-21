/* -------------------------------------------------------------------------- */
/*  Shared JSON-LD Schema Helpers                                             */
/*  Centralizes @id references and entity definitions for the @graph pattern  */
/* -------------------------------------------------------------------------- */

export const SITE_URL = 'https://aigiscode.com';
export const ORG_NAME = 'AigisCode';
export const PARENT_ORG = 'Draivix';
export const FOUNDER_NAME = 'David Strejc';
export const LOGO_URL = `${SITE_URL}/favicon.svg`;

/* ---- Stable @id constants ---- */
export const IDS = {
  organization: `${SITE_URL}/#organization`,
  website: `${SITE_URL}/#website`,
  software: `${SITE_URL}/#software`,
  sourceCode: `${SITE_URL}/#sourcecode`,
  founder: `${SITE_URL}/#founder`,
} as const;

/* ---- Reusable entity references (use @id only, no duplication) ---- */
export const orgRef = () => ({ '@id': IDS.organization });
export const websiteRef = () => ({ '@id': IDS.website });
export const softwareRef = () => ({ '@id': IDS.software });
export const founderRef = () => ({ '@id': IDS.founder });

/* ---- Shared Organization entity (for pages that need the full entity) ---- */
export function organizationEntity() {
  return {
    '@type': 'Organization',
    '@id': IDS.organization,
    name: ORG_NAME,
    url: SITE_URL,
    logo: {
      '@type': 'ImageObject',
      url: LOGO_URL,
      width: '512',
      height: '512',
    },
    sameAs: [
      'https://github.com/Draivix/aigiscode',
      'https://x.com/aigiscode',
      'https://pypi.org/project/aigiscode/',
    ],
    parentOrganization: {
      '@type': 'Organization',
      name: PARENT_ORG,
      url: 'https://draivix.com',
    },
    founder: founderRef(),
  };
}

/* ---- Shared Person entity for the founder / primary author ---- */
export function founderEntity() {
  return {
    '@type': 'Person',
    '@id': IDS.founder,
    name: FOUNDER_NAME,
    jobTitle: 'Founder & CEO',
    url: `${SITE_URL}/about`,
    sameAs: [
      'https://github.com/david-strejc',
      'https://x.com/aigiscode',
    ],
  };
}

/* ---- WebPage entity for any page ---- */
export function webPageEntity(url: string, name: string, description: string) {
  return {
    '@type': 'WebPage',
    '@id': `${url}#webpage`,
    url,
    name,
    description,
    isPartOf: websiteRef(),
    about: softwareRef(),
    inLanguage: ['en', 'cs', 'fr', 'es', 'zh-CN', 'hi', 'pt-BR', 'pl', 'ar', 'bn-BD'],
  };
}

/* ---- TechArticle for feature/language/platform/integration detail pages ---- */
export function techArticleEntity(opts: {
  url: string;
  headline: string;
  description: string;
  datePublished: string;
  dateModified: string;
  keywords: string[];
}) {
  return {
    '@type': 'TechArticle',
    '@id': `${opts.url}#article`,
    headline: opts.headline,
    description: opts.description,
    url: opts.url,
    datePublished: opts.datePublished,
    dateModified: opts.dateModified,
    keywords: opts.keywords.join(', '),
    author: orgRef(),
    publisher: orgRef(),
    mainEntityOfPage: { '@type': 'WebPage', '@id': `${opts.url}#webpage` },
    inLanguage: ['en', 'cs', 'fr', 'es', 'zh-CN', 'hi', 'pt-BR', 'pl', 'ar', 'bn-BD'],
    proficiencyLevel: 'Expert',
  };
}

/* ---- Article for use case pages ---- */
export function articleEntity(opts: {
  url: string;
  headline: string;
  description: string;
  datePublished: string;
  dateModified: string;
  keywords: string[];
}) {
  return {
    '@type': 'Article',
    '@id': `${opts.url}#article`,
    headline: opts.headline,
    description: opts.description,
    url: opts.url,
    datePublished: opts.datePublished,
    dateModified: opts.dateModified,
    keywords: opts.keywords.join(', '),
    author: orgRef(),
    publisher: orgRef(),
    mainEntityOfPage: { '@type': 'WebPage', '@id': `${opts.url}#webpage` },
    inLanguage: ['en', 'cs', 'fr', 'es', 'zh-CN', 'hi', 'pt-BR', 'pl', 'ar', 'bn-BD'],
  };
}

/* ---- BlogPosting for blog post pages ---- */
export function blogPostingEntity(opts: {
  url: string;
  headline: string;
  description: string;
  datePublished: string;
  dateModified: string;
  authorName: string;
  wordCount?: number;
  tags?: string[];
}) {
  return {
    '@type': 'BlogPosting',
    '@id': `${opts.url}#article`,
    headline: opts.headline,
    description: opts.description,
    url: opts.url,
    datePublished: opts.datePublished,
    dateModified: opts.dateModified,
    wordCount: opts.wordCount,
    author: founderRef(),
    publisher: orgRef(),
    mainEntityOfPage: { '@type': 'WebPage', '@id': `${opts.url}#webpage` },
    inLanguage: ['en', 'cs', 'fr', 'es', 'zh-CN', 'hi', 'pt-BR', 'pl', 'ar', 'bn-BD'],
    ...(opts.tags && opts.tags.length > 0
      ? { keywords: opts.tags.join(', ') }
      : {}),
  };
}

/* ---- CollectionPage for hub/listing pages ---- */
export function collectionPageEntity(opts: {
  url: string;
  name: string;
  description: string;
  items: { name: string; url: string }[];
}) {
  return {
    '@type': 'CollectionPage',
    '@id': `${opts.url}#webpage`,
    url: opts.url,
    name: opts.name,
    description: opts.description,
    isPartOf: websiteRef(),
    about: softwareRef(),
    inLanguage: ['en', 'cs', 'fr', 'es', 'zh-CN', 'hi', 'pt-BR', 'pl', 'ar', 'bn-BD'],
    mainEntity: {
      '@type': 'ItemList',
      numberOfItems: opts.items.length,
      itemListElement: opts.items.map((item, i) => ({
        '@type': 'ListItem',
        position: i + 1,
        name: item.name,
        url: item.url,
      })),
    },
  };
}

/* ---- HowTo entity for docs page ---- */
export function howToEntity(opts: {
  name: string;
  description: string;
  steps: { name: string; text: string; url: string }[];
}) {
  return {
    '@type': 'HowTo',
    '@id': `${SITE_URL}/docs#howto`,
    name: opts.name,
    description: opts.description,
    step: opts.steps.map((step) => ({
      '@type': 'HowToStep',
      name: step.name,
      text: step.text,
      url: step.url,
    })),
  };
}

/* ---- SoftwareApplication feature list for feature pages ---- */
export function softwareFeatureEntity(opts: {
  url: string;
  name: string;
  description: string;
  featureList: string[];
}) {
  return {
    '@type': 'SoftwareApplication',
    '@id': `${opts.url}#softwarefeature`,
    name: `AigisCode - ${opts.name}`,
    description: opts.description,
    applicationCategory: 'DeveloperApplication',
    operatingSystem: 'Cross-platform',
    featureList: opts.featureList,
    url: opts.url,
    offers: {
      '@type': 'Offer',
      price: '0',
      priceCurrency: 'USD',
    },
    author: orgRef(),
  };
}

/* ---- Build a full @graph JSON-LD block ---- */
export function buildGraphJsonLd(entities: object[]): object {
  return {
    '@context': 'https://schema.org',
    '@graph': entities,
  };
}
