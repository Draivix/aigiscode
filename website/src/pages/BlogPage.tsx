import { useState, useMemo, useEffect, useRef, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { animate, stagger, createTimeline } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';
import { MagnifyingGlass, Funnel } from '@phosphor-icons/react';
import SEO from '@/components/SEO';
import Breadcrumbs from '@/components/Breadcrumbs';
import BlogCard from '@/components/BlogCard';
import { blogPosts, getAllTags } from '@/content/blog-posts';
import {
  SITE_URL,
  buildGraphJsonLd,
  collectionPageEntity,
} from '@/content/seo-schema';

/* -------------------------------------------------------------------------- */
/*  Constants                                                                 */
/* -------------------------------------------------------------------------- */

const ALL_TAGS = getAllTags();

/* -------------------------------------------------------------------------- */
/*  Tag Cloud                                                                 */
/* -------------------------------------------------------------------------- */

function TagCloud({
  tags,
  activeTag,
  onTagClick,
}: {
  tags: string[];
  activeTag: string | null;
  onTagClick: (tag: string | null) => void;
}) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-wrap items-center gap-2">
      <span className="text-xs font-medium text-zinc-500 dark:text-zinc-500 uppercase tracking-wider mr-1 flex items-center gap-1">
        <Funnel size={14} weight="duotone" />
        {t('blog.tags')}
      </span>

      {/* "All" button */}
      <button
        onClick={() => onTagClick(null)}
        className={`px-3 py-1 text-xs font-medium rounded-full border transition-colors ${
          activeTag === null
            ? 'bg-indigo-500 text-white border-indigo-500'
            : 'bg-white/50 dark:bg-white/5 text-zinc-600 dark:text-zinc-400 border-zinc-200/50 dark:border-white/10 hover:border-indigo-500/30 hover:text-indigo-600 dark:hover:text-indigo-400'
        }`}
      >
        All
      </button>

      {tags.map((tag) => (
        <button
          key={tag}
          onClick={() => onTagClick(activeTag === tag ? null : tag)}
          className={`px-3 py-1 text-xs font-medium rounded-full border transition-colors ${
            activeTag === tag
              ? 'bg-indigo-500 text-white border-indigo-500'
              : 'bg-white/50 dark:bg-white/5 text-zinc-600 dark:text-zinc-400 border-zinc-200/50 dark:border-white/10 hover:border-indigo-500/30 hover:text-indigo-600 dark:hover:text-indigo-400'
          }`}
        >
          {tag}
        </button>
      ))}
    </div>
  );
}

/* -------------------------------------------------------------------------- */
/*  BlogPage                                                                  */
/* -------------------------------------------------------------------------- */

export default function BlogPage() {
  const { t, i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';

  const [activeTag, setActiveTag] = useState<string | null>(null);
  const [search, setSearch] = useState('');

  /* Refs for anime.js animations */
  const heroRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<HTMLDivElement>(null);
  const hasAnimatedGrid = useRef(false);
  const observerRef = useRef<IntersectionObserver | null>(null);

  /* Filter posts by tag and search query */
  const filteredPosts = useMemo(() => {
    let posts = [...blogPosts];

    if (activeTag) {
      posts = posts.filter((p) => p.tags.includes(activeTag));
    }

    if (search.trim()) {
      const q = search.toLowerCase();
      posts = posts.filter((p) => {
        const title = (p.title[lang] || p.title.en).toLowerCase();
        const desc = (p.description[lang] || p.description.en).toLowerCase();
        const tags = p.tags.join(' ').toLowerCase();
        return title.includes(q) || desc.includes(q) || tags.includes(q);
      });
    }

    /* Sort by date descending */
    posts.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());

    return posts;
  }, [activeTag, search, lang]);

  /* Format date for display */
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

  /* ---- Hero + Search timeline animation ---- */
  useEffect(() => {
    if (!heroRef.current || !searchRef.current) return;

    if (prefersReducedMotion()) {
      heroRef.current.style.opacity = '1';
      heroRef.current.style.transform = 'none';
      searchRef.current.style.opacity = '1';
      searchRef.current.style.transform = 'none';
      return;
    }

    const tl = createTimeline({ defaults: { ease: 'outQuart' } })
      .add(heroRef.current, {
        opacity: [0, 1],
        translateY: [30, 0],
        duration: 600,
      })
      .add(searchRef.current, {
        opacity: [0, 1],
        translateY: [20, 0],
        duration: 500,
      }, '-=300');

    return () => { tl.pause(); };
  }, []);

  /* ---- Grid stagger animation handler ---- */
  const animateGrid = useCallback(() => {
    if (!gridRef.current) return;

    const cards = gridRef.current.querySelectorAll('[data-blog-card]');
    if (cards.length === 0) return;

    if (prefersReducedMotion()) {
      cards.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    animate(cards, {
      opacity: [0, 1],
      scale: [0.9, 1],
      translateY: [25, 0],
      duration: 450,
      delay: stagger(70, { from: 'first' }),
      ease: 'outQuart',
    });
  }, []);

  /* ---- IntersectionObserver for grid — initial setup ---- */
  useEffect(() => {
    if (!gridRef.current) return;

    if (prefersReducedMotion()) {
      gridRef.current.querySelectorAll('[data-blog-card]').forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    observerRef.current = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimatedGrid.current) {
          hasAnimatedGrid.current = true;
          animateGrid();
          observerRef.current?.disconnect();
        }
      },
      { threshold: 0.1, rootMargin: '-60px' },
    );

    observerRef.current.observe(gridRef.current);

    return () => { observerRef.current?.disconnect(); };
  }, [animateGrid]);

  /* ---- Re-trigger animation when filteredPosts changes ---- */
  useEffect(() => {
    if (!gridRef.current || prefersReducedMotion()) {
      if (gridRef.current) {
        gridRef.current.querySelectorAll('[data-blog-card]').forEach((el) => {
          (el as HTMLElement).style.opacity = '1';
        });
      }
      return;
    }

    // Reset animation state
    hasAnimatedGrid.current = false;

    // Reset card opacity so the animation replays
    gridRef.current.querySelectorAll('[data-blog-card]').forEach((el) => {
      (el as HTMLElement).style.opacity = '0';
      (el as HTMLElement).style.transform = '';
    });

    // Disconnect existing observer
    observerRef.current?.disconnect();

    // Re-observe
    observerRef.current = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimatedGrid.current) {
          hasAnimatedGrid.current = true;
          animateGrid();
          observerRef.current?.disconnect();
        }
      },
      { threshold: 0.1, rootMargin: '-60px' },
    );

    observerRef.current.observe(gridRef.current);

    return () => { observerRef.current?.disconnect(); };
  }, [filteredPosts, animateGrid]);

  return (
    <>
      <SEO
        title={`${t('blog.title')} | AigisCode`}
        description={t('blog.subtitle')}
        canonical={`${SITE_URL}/blog`}
        breadcrumbs={[
          { name: 'Home', url: SITE_URL },
          { name: t('blog.title'), url: `${SITE_URL}/blog` },
        ]}
        jsonLd={buildGraphJsonLd([
          collectionPageEntity({
            url: `${SITE_URL}/blog`,
            name: 'AigisCode Blog',
            description: 'Insights on AI-powered code analysis, static analysis best practices, code quality, and developer workflows.',
            items: blogPosts.map((p) => ({
              name: p.title.en,
              url: `${SITE_URL}/blog/${p.slug}`,
            })),
          }),
        ])}
      />

      <section className="py-20 md:py-28">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          {/* Breadcrumbs */}
          <Breadcrumbs
            items={[
              { label: 'Home', href: '/' },
              { label: t('blog.title') },
            ]}
          />

          {/* Hero */}
          <div
            ref={heroRef}
            className="mt-8 mb-12 opacity-0"
          >
            <h1 className="text-4xl md:text-5xl lg:text-6xl font-display font-bold tracking-tight text-zinc-900 dark:text-white">
              {t('blog.title')}
            </h1>
            <p className="mt-4 text-lg text-zinc-600 dark:text-zinc-400 max-w-2xl">
              {t('blog.subtitle')}
            </p>
          </div>

          {/* Search + Tag Filter */}
          <div
            ref={searchRef}
            className="mb-10 space-y-4 opacity-0"
          >
            {/* Search */}
            <div className="relative max-w-md">
              <MagnifyingGlass
                size={18}
                className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-400"
                weight="duotone"
              />
              <input
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Search articles..."
                className="w-full pl-10 pr-4 py-2.5 rounded-xl border border-zinc-200/50 dark:border-white/10 bg-white/50 dark:bg-white/5 backdrop-blur-sm text-sm text-zinc-900 dark:text-zinc-100 placeholder-zinc-400 dark:placeholder-zinc-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/30 focus:border-indigo-500/30 transition-colors"
              />
            </div>

            {/* Tags */}
            <TagCloud tags={ALL_TAGS} activeTag={activeTag} onTagClick={setActiveTag} />
          </div>

          {/* Post Grid */}
          {filteredPosts.length > 0 ? (
            <div ref={gridRef} className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {filteredPosts.map((post) => (
                <div key={post.slug} data-blog-card className="opacity-0">
                  <BlogCard
                    slug={post.slug}
                    title={post.title[lang] || post.title.en}
                    description={post.description[lang] || post.description.en}
                    date={formatDate(post.date)}
                    readTime={`${post.readTime} ${t('blog.minRead')}`}
                    tags={post.tags}
                    image={post.image}
                  />
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-20">
              <p className="text-zinc-500 dark:text-zinc-500 text-lg">
                No articles match your search.
              </p>
              <button
                onClick={() => {
                  setSearch('');
                  setActiveTag(null);
                }}
                className="mt-4 text-sm text-indigo-500 hover:text-indigo-400 transition-colors"
              >
                Clear filters
              </button>
            </div>
          )}
        </div>
      </section>
    </>
  );
}
