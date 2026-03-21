import { LocalizedLink as Link } from '@/lib/i18n-routing';
import { useTranslation } from 'react-i18next';
import { House, MagnifyingGlass } from '@phosphor-icons/react';
import SEO from '../components/SEO';

export default function NotFoundPage() {
  const { t } = useTranslation();

  return (
    <>
      <SEO
        title="404 — Page Not Found | AigisCode"
        description="The page you're looking for doesn't exist or has been moved."
        canonical="https://aigiscode.com/404"
        noindex
      />

      <section className="min-h-[60vh] flex items-center justify-center px-4">
        <div className="text-center max-w-md">
          <p className="text-7xl font-display font-bold bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500 mb-4">
            404
          </p>
          <h1 className="text-2xl font-semibold text-zinc-900 dark:text-white mb-3">
            {t('notFound.title', 'Page Not Found')}
          </h1>
          <p className="text-zinc-500 mb-8">
            {t('notFound.description', "The page you're looking for doesn't exist or has been moved.")}
          </p>
          <div className="flex items-center justify-center gap-4">
            <Link
              to="/"
              className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg bg-zinc-900 dark:bg-white text-white dark:text-black text-sm font-medium hover:scale-105 transition-transform"
            >
              <House size={16} weight="bold" />
              {t('notFound.backHome', 'Back to Home')}
            </Link>
            <Link
              to="/docs"
              className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg border border-zinc-200 dark:border-white/10 text-sm font-medium hover:bg-zinc-100 dark:hover:bg-white/5 transition-colors"
            >
              <MagnifyingGlass size={16} />
              {t('notFound.viewDocs', 'View Docs')}
            </Link>
          </div>
        </div>
      </section>
    </>
  );
}
