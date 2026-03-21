import { LocalizedLink as Link } from '@/lib/i18n-routing';
import { useTranslation } from 'react-i18next';
import { CaretRight, House } from '@phosphor-icons/react';

/* -------------------------------------------------------------------------- */
/*  Types                                                                     */
/* -------------------------------------------------------------------------- */

export interface BreadcrumbItem {
  label: string;
  href?: string;
}

interface BreadcrumbsProps {
  items: BreadcrumbItem[];
}

/* -------------------------------------------------------------------------- */
/*  Breadcrumbs                                                               */
/* -------------------------------------------------------------------------- */

export default function Breadcrumbs({ items }: BreadcrumbsProps) {
  const { t } = useTranslation();
  return (
    <nav aria-label="Breadcrumb" className="mb-6">
      <ol className="flex flex-wrap items-center gap-1.5 text-sm text-zinc-500 dark:text-zinc-400">
        {/* Home */}
        <li className="flex items-center gap-1.5">
          <Link
            to="/"
            className="flex items-center gap-1 hover:text-zinc-900 dark:hover:text-white transition-colors"
          >
            <House size={14} weight="bold" />
            <span>{t('breadcrumbs.home', 'Home')}</span>
          </Link>
        </li>

        {/* Remaining items */}
        {items.map((item, index) => {
          const isLast = index === items.length - 1;

          return (
            <li key={index} className="flex items-center gap-1.5">
              <CaretRight size={12} className="text-zinc-400 dark:text-zinc-600 flex-shrink-0" />
              {isLast || !item.href ? (
                <span className="text-zinc-900 dark:text-white font-medium truncate max-w-[200px] sm:max-w-none">
                  {item.label}
                </span>
              ) : (
                <Link
                  to={item.href}
                  className="hover:text-zinc-900 dark:hover:text-white transition-colors truncate max-w-[200px] sm:max-w-none"
                >
                  {item.label}
                </Link>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
