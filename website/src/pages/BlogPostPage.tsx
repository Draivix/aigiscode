import { useState, useEffect, useMemo, useRef } from 'react';
import { useParams } from 'react-router-dom';
import { LocalizedLink as Link } from '@/lib/i18n-routing';
import { useTranslation } from 'react-i18next';
import { animate, stagger } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';
import {
  CalendarBlank,
  Clock,
  User,
  LinkSimple,
  Check,
  XLogo,
  ArrowLeft,
  List as ListIcon,
  CaretRight,
} from '@phosphor-icons/react';
import SEO from '@/components/SEO';
import Breadcrumbs from '@/components/Breadcrumbs';
import BlogCard from '@/components/BlogCard';
import { getBlogPost, getRelatedPosts } from '@/content/blog-posts';
import type { BlogPost } from '@/content/blog-posts';
import {
  SITE_URL,
  buildGraphJsonLd,
  blogPostingEntity,
  founderEntity,
  webPageEntity,
} from '@/content/seo-schema';

/* -------------------------------------------------------------------------- */
/*  Constants                                                                 */
/* -------------------------------------------------------------------------- */

/* -------------------------------------------------------------------------- */
/*  Table of Contents                                                         */
/* -------------------------------------------------------------------------- */

interface TocItem {
  id: string;
  text: string;
  level: number;
}

function extractToc(html: string): TocItem[] {
  const items: TocItem[] = [];
  const regex = /<h([23])\s+id="([^"]+)"[^>]*>(.*?)<\/h[23]>/gi;
  let match;
  while ((match = regex.exec(html)) !== null) {
    items.push({
      level: parseInt(match[1], 10),
      id: match[2],
      text: match[3].replace(/<[^>]*>/g, ''),
    });
  }
  return items;
}

function TableOfContents({
  items,
  activeId,
  tocRef,
}: {
  items: TocItem[];
  activeId: string;
  tocRef?: React.RefObject<HTMLElement | null>;
}) {
  const { t } = useTranslation();

  if (items.length === 0) return null;

  return (
    <nav
      ref={tocRef}
      aria-label={t('common.tableOfContents')}
      className="sticky top-24 space-y-1"
    >
      <h4 className="text-xs font-semibold uppercase tracking-wider text-zinc-500 dark:text-zinc-500 mb-3 flex items-center gap-1.5">
        <ListIcon size={14} weight="bold" />
        {t('common.tableOfContents')}
      </h4>
      <ul className="space-y-1 text-sm border-l border-zinc-200/50 dark:border-white/10">
        {items.map((item) => (
          <li key={item.id} data-toc-item className="opacity-0">
            <a
              href={`#${item.id}`}
              className={`block py-1 transition-colors ${
                item.level === 3 ? 'pl-6' : 'pl-3'
              } ${
                activeId === item.id
                  ? 'text-indigo-600 dark:text-indigo-400 border-l-2 border-indigo-500 -ml-[1px] font-medium'
                  : 'text-zinc-500 dark:text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-300'
              }`}
            >
              {item.text}
            </a>
          </li>
        ))}
      </ul>
    </nav>
  );
}

/* -------------------------------------------------------------------------- */
/*  Share Buttons                                                             */
/* -------------------------------------------------------------------------- */

function ShareButtons({ url, title }: { url: string; title: string }) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  const handleCopyLink = async () => {
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      /* clipboard not available */
    }
  };

  const twitterUrl = `https://twitter.com/intent/tweet?text=${encodeURIComponent(title)}&url=${encodeURIComponent(url)}`;

  return (
    <div className="flex items-center gap-2">
      <span className="text-xs font-medium text-zinc-500 dark:text-zinc-500 uppercase tracking-wider mr-1">
        {t('common.share')}
      </span>

      {/* Copy link */}
      <button
        onClick={handleCopyLink}
        className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg border border-zinc-200/50 dark:border-white/10 bg-white/50 dark:bg-white/5 text-zinc-600 dark:text-zinc-400 hover:text-indigo-600 dark:hover:text-indigo-400 hover:border-indigo-500/30 transition-colors"
        aria-label="Copy link"
      >
        {copied ? (
          <>
            <Check size={14} weight="bold" className="text-emerald-500" />
            Copied
          </>
        ) : (
          <>
            <LinkSimple size={14} weight="bold" />
            Copy link
          </>
        )}
      </button>

      {/* Twitter / X */}
      <a
        href={twitterUrl}
        target="_blank"
        rel="noopener noreferrer"
        className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg border border-zinc-200/50 dark:border-white/10 bg-white/50 dark:bg-white/5 text-zinc-600 dark:text-zinc-400 hover:text-indigo-600 dark:hover:text-indigo-400 hover:border-indigo-500/30 transition-colors"
        aria-label="Share on X"
      >
        <XLogo size={14} weight="bold" />
        Post
      </a>
    </div>
  );
}

/* -------------------------------------------------------------------------- */
/*  Related Posts                                                             */
/* -------------------------------------------------------------------------- */

function RelatedPosts({
  post,
  relatedRef,
}: {
  post: BlogPost;
  relatedRef: React.RefObject<HTMLDivElement | null>;
}) {
  const { t, i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';
  const related = getRelatedPosts(post);

  if (related.length === 0) return null;

  const formatDate = (iso: string) => {
    try {
      return new Date(iso).toLocaleDateString(lang === 'en' ? 'en-US' : lang, {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      });
    } catch {
      return iso;
    }
  };

  return (
    <section className="mt-16 pt-12 border-t border-zinc-200/50 dark:border-white/10">
      <h2 className="text-2xl font-display font-bold text-zinc-900 dark:text-white mb-8">
        {t('blog.relatedPosts')}
      </h2>
      <div ref={relatedRef} className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {related.map((r) => (
          <div key={r.slug} data-related-post className="opacity-0">
            <BlogCard
              slug={r.slug}
              title={r.title[lang] || r.title.en}
              description={r.description[lang] || r.description.en}
              date={formatDate(r.date)}
              readTime={`${r.readTime} ${t('blog.minRead')}`}
              tags={r.tags}
              image={r.image}
            />
          </div>
        ))}
      </div>
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Not Found                                                                 */
/* -------------------------------------------------------------------------- */

function PostNotFound() {
  const { t } = useTranslation();

  return (
    <section className="py-32 text-center">
      <div className="max-w-2xl mx-auto px-4">
        <h1 className="text-4xl font-display font-bold text-zinc-900 dark:text-white mb-4">
          Article Not Found
        </h1>
        <p className="text-zinc-500 dark:text-zinc-400 mb-8">
          The blog post you are looking for does not exist or has been moved.
        </p>
        <Link
          to="/blog"
          className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-zinc-900 dark:bg-white text-white dark:text-black text-sm font-semibold hover:scale-105 transition-transform"
        >
          <ArrowLeft size={16} weight="bold" />
          {t('blog.backToBlog')}
        </Link>
      </div>
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/*  BlogPostPage                                                              */
/* -------------------------------------------------------------------------- */

export default function BlogPostPage() {
  const { slug } = useParams<{ slug: string }>();
  const { t, i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';

  const post = slug ? getBlogPost(slug) : undefined;

  /* Resolve localized fields */
  const title = post ? (post.title[lang] || post.title.en) : '';
  const description = post ? (post.description[lang] || post.description.en) : '';
  const metaDescription = post ? (post.metaDescription[lang] || post.metaDescription.en) : '';
  const content = post ? (post.content[lang] || post.content.en) : '';

  /* Table of contents */
  const tocItems = useMemo(() => (content ? extractToc(content) : []), [content]);

  /* Active heading tracking for TOC */
  const [activeId, setActiveId] = useState('');

  useEffect(() => {
    if (tocItems.length === 0) return;

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setActiveId(entry.target.id);
          }
        }
      },
      { rootMargin: '-80px 0px -60% 0px', threshold: 0 },
    );

    for (const item of tocItems) {
      const el = document.getElementById(item.id);
      if (el) observer.observe(el);
    }

    return () => observer.disconnect();
  }, [tocItems]);

  /* ---- anime.js refs ---- */
  const heroRef = useRef<HTMLElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const contentAnimated = useRef(false);
  const tocNavRef = useRef<HTMLElement>(null);
  const tocAnimated = useRef(false);
  const relatedGridRef = useRef<HTMLDivElement>(null);
  const relatedAnimated = useRef(false);

  /* ---- Hero entrance animation ---- */
  useEffect(() => {
    if (!heroRef.current) return;
    if (prefersReducedMotion()) {
      heroRef.current.style.opacity = '1';
      heroRef.current.style.transform = 'none';
      return;
    }

    const anim = animate(heroRef.current, {
      opacity: [0, 1],
      translateY: [30, 0],
      duration: 600,
      ease: 'outQuart',
    });

    return () => { anim.pause(); };
  }, [slug]);

  /* ---- Article content fade-in with IntersectionObserver ---- */
  useEffect(() => {
    if (!contentRef.current) return;
    contentAnimated.current = false;

    if (prefersReducedMotion()) {
      contentRef.current.style.opacity = '1';
      return;
    }

    const el = contentRef.current;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !contentAnimated.current) {
          contentAnimated.current = true;
          animate(el, {
            opacity: [0, 1],
            translateY: [20, 0],
            duration: 500,
            ease: 'outQuart',
          });
          observer.disconnect();
        }
      },
      { threshold: 0.05, rootMargin: '-40px' },
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, [slug]);

  /* ---- TOC items stagger animation ---- */
  useEffect(() => {
    if (!tocNavRef.current) return;
    tocAnimated.current = false;

    const items = tocNavRef.current.querySelectorAll('[data-toc-item]');
    if (items.length === 0) return;

    if (prefersReducedMotion()) {
      items.forEach((el) => { (el as HTMLElement).style.opacity = '1'; });
      return;
    }

    // Delay slightly to let the hero animation finish first
    const timer = setTimeout(() => {
      if (tocAnimated.current) return;
      tocAnimated.current = true;
      animate(items, {
        opacity: [0, 1],
        translateX: [-10, 0],
        duration: 350,
        delay: stagger(40),
        ease: 'outQuart',
      });
    }, 700);

    return () => clearTimeout(timer);
  }, [slug, tocItems]);

  /* ---- Related posts stagger on scroll ---- */
  useEffect(() => {
    if (!relatedGridRef.current) return;
    relatedAnimated.current = false;

    const items = relatedGridRef.current.querySelectorAll('[data-related-post]');
    if (items.length === 0) return;

    if (prefersReducedMotion()) {
      items.forEach((el) => { (el as HTMLElement).style.opacity = '1'; });
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !relatedAnimated.current) {
          relatedAnimated.current = true;
          animate(items, {
            opacity: [0, 1],
            translateY: [20, 0],
            duration: 450,
            delay: stagger(80),
            ease: 'outQuart',
          });
          observer.disconnect();
        }
      },
      { threshold: 0.1, rootMargin: '-60px' },
    );

    observer.observe(relatedGridRef.current);
    return () => observer.disconnect();
  }, [slug]);

  /* Format date */
  const formatDate = (iso: string) => {
    try {
      return new Date(iso).toLocaleDateString(lang === 'en' ? 'en-US' : lang, {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      });
    } catch {
      return iso;
    }
  };

  /* ----------- Not found ----------- */
  if (!post) {
    return (
      <>
        <SEO
          title="Article Not Found | AigisCode"
          description="The requested blog post could not be found."
          canonical={`${SITE_URL}/blog`}
        />
        <PostNotFound />
      </>
    );
  }

  /* ----------- JSON-LD ----------- */
  const canonicalUrl = `${SITE_URL}/blog/${post.slug}`;

  const graphJsonLd = buildGraphJsonLd([
    webPageEntity(canonicalUrl, `${post.title.en} - AigisCode Blog`, post.metaDescription.en),
    founderEntity(),
    blogPostingEntity({
      url: canonicalUrl,
      headline: post.title.en,
      description: post.metaDescription.en,
      datePublished: post.date,
      dateModified: post.date,
      authorName: post.author.name,
      tags: post.tags,
      wordCount: Math.round((post.content.en || '').replace(/<[^>]+>/g, '').split(/\s+/).length),
    }),
  ]);

  return (
    <>
      <SEO
        title={`${title} | AigisCode Blog`}
        description={metaDescription}
        canonical={canonicalUrl}
        article={{
          publishedTime: post.date,
          modifiedTime: post.date,
          author: post.author.name,
          tags: post.tags,
        }}
        breadcrumbs={[
          { name: 'Home', url: SITE_URL },
          { name: t('blog.title'), url: `${SITE_URL}/blog` },
          { name: title, url: canonicalUrl },
        ]}
        jsonLd={graphJsonLd}
      />

      <article className="py-20 md:py-28">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          {/* Breadcrumbs */}
          <Breadcrumbs
            items={[
              { label: 'Home', href: '/' },
              { label: t('blog.title'), href: '/blog' },
              { label: title },
            ]}
          />

          {/* Back link */}
          <Link
            to="/blog"
            className="inline-flex items-center gap-1.5 mt-6 text-sm text-zinc-500 dark:text-zinc-500 hover:text-indigo-500 dark:hover:text-indigo-400 transition-colors"
          >
            <ArrowLeft size={14} weight="bold" />
            {t('blog.backToBlog')}
          </Link>

          {/* Header */}
          <header
            ref={heroRef}
            className="mt-6 mb-12 max-w-3xl opacity-0"
          >
            {/* Tags */}
            <div className="flex flex-wrap gap-1.5 mb-4">
              {post.tags.map((tag) => (
                <Link
                  key={tag}
                  to={`/blog?tag=${encodeURIComponent(tag)}`}
                  className="inline-block px-2.5 py-0.5 text-xs font-medium rounded-full bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/10 dark:border-indigo-500/20 hover:bg-indigo-500/20 transition-colors"
                >
                  {tag}
                </Link>
              ))}
            </div>

            {/* Title */}
            <h1 className="text-3xl md:text-4xl lg:text-5xl font-display font-bold tracking-tight text-zinc-900 dark:text-white leading-tight">
              {title}
            </h1>

            {/* Meta */}
            <div className="mt-6 flex flex-wrap items-center gap-4 text-sm text-zinc-500 dark:text-zinc-500">
              <span className="flex items-center gap-1.5">
                <User size={16} weight="duotone" />
                <span>{post.author.name}</span>
                <span className="text-zinc-400 dark:text-zinc-600">
                  <CaretRight size={10} weight="bold" />
                </span>
                <span className="text-zinc-400 dark:text-zinc-600">{post.author.role}</span>
              </span>
              <span className="flex items-center gap-1.5">
                <CalendarBlank size={16} weight="duotone" />
                {formatDate(post.date)}
              </span>
              <span className="flex items-center gap-1.5">
                <Clock size={16} weight="duotone" />
                {post.readTime} {t('blog.minRead')}
              </span>
            </div>

            {/* Share */}
            <div className="mt-6">
              <ShareButtons url={canonicalUrl} title={title} />
            </div>
          </header>

          {/* Content + TOC Layout */}
          <div className="flex gap-12 items-start">
            {/* Article Body */}
            <div
              ref={contentRef}
              className="flex-1 min-w-0 max-w-3xl opacity-0"
            >
              {/* Prose content */}
              <div
                className="
                  prose prose-zinc dark:prose-invert
                  prose-headings:font-display prose-headings:font-semibold prose-headings:tracking-tight
                  prose-h2:text-2xl prose-h2:mt-12 prose-h2:mb-4
                  prose-h3:text-xl prose-h3:mt-8 prose-h3:mb-3
                  prose-p:leading-relaxed prose-p:text-zinc-600 prose-p:dark:text-zinc-400
                  prose-a:text-indigo-600 prose-a:dark:text-indigo-400 prose-a:no-underline hover:prose-a:underline
                  prose-strong:text-zinc-900 prose-strong:dark:text-zinc-200
                  prose-code:text-indigo-600 prose-code:dark:text-indigo-400 prose-code:bg-indigo-500/10 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm prose-code:before:content-none prose-code:after:content-none
                  prose-pre:bg-zinc-900 prose-pre:dark:bg-zinc-950 prose-pre:border prose-pre:border-zinc-800 prose-pre:dark:border-zinc-800/60 prose-pre:rounded-xl
                  prose-table:border-collapse
                  prose-th:bg-zinc-100 prose-th:dark:bg-zinc-800/50 prose-th:px-4 prose-th:py-2 prose-th:text-left prose-th:text-sm prose-th:font-semibold prose-th:border prose-th:border-zinc-200 prose-th:dark:border-zinc-700
                  prose-td:px-4 prose-td:py-2 prose-td:text-sm prose-td:border prose-td:border-zinc-200 prose-td:dark:border-zinc-700
                  max-w-none
                "
                dangerouslySetInnerHTML={{ __html: content }}
              />

              {/* Bottom share */}
              <div className="mt-12 pt-8 border-t border-zinc-200/50 dark:border-white/10">
                <ShareButtons url={canonicalUrl} title={title} />
              </div>
            </div>

            {/* Desktop TOC Sidebar */}
            {tocItems.length > 0 && (
              <aside className="hidden xl:block w-64 flex-shrink-0">
                <TableOfContents items={tocItems} activeId={activeId} tocRef={tocNavRef} />
              </aside>
            )}
          </div>

          {/* Related Posts */}
          <RelatedPosts post={post} relatedRef={relatedGridRef} />
        </div>
      </article>
    </>
  );
}
