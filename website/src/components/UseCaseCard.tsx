import { LocalizedLink as Link } from '@/lib/i18n-routing';
import { useTranslation } from 'react-i18next';
import { ArrowRight } from '@phosphor-icons/react';
import type { UseCase } from '@/content/use-cases';

/* -------------------------------------------------------------------------- */
/*  Types                                                                     */
/* -------------------------------------------------------------------------- */

interface UseCaseCardProps {
  useCase: UseCase;
}

/* -------------------------------------------------------------------------- */
/*  UseCaseCard                                                               */
/* -------------------------------------------------------------------------- */

export default function UseCaseCard({ useCase }: UseCaseCardProps) {
  const { i18n, t } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';

  const title = useCase.title[lang] || useCase.title.en;
  const description = useCase.shortDescription[lang] || useCase.shortDescription.en;

  return (
    <Link
      to={`/use-cases/${useCase.slug}`}
      className="group relative flex flex-col rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 overflow-hidden transition-all duration-300 hover:shadow-lg hover:shadow-indigo-500/5 hover:border-indigo-500/20 dark:hover:border-indigo-500/20"
    >
      {/* Hover glow */}
      <div className="absolute -inset-1 rounded-2xl bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-pink-500/20 opacity-0 group-hover:opacity-100 blur-xl transition-opacity duration-500 pointer-events-none" />

      <div className="relative flex flex-col flex-1">
        {/* Category badge */}
        <span className="inline-flex self-start items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 mb-4">
          {useCase.category}
        </span>

        {/* Title */}
        <h3 className="font-display font-semibold text-lg mb-2 text-zinc-900 dark:text-white group-hover:text-indigo-600 dark:group-hover:text-indigo-400 transition-colors">
          {title}
        </h3>

        {/* Description */}
        <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed flex-1">
          {description}
        </p>

        {/* Features preview */}
        <div className="mt-4 flex flex-wrap gap-1.5">
          {useCase.features.slice(0, 3).map((feature) => (
            <span
              key={feature}
              className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-zinc-100 dark:bg-white/5 text-zinc-500 dark:text-zinc-500"
            >
              {feature}
            </span>
          ))}
          {useCase.features.length > 3 && (
            <span className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-zinc-100 dark:bg-white/5 text-zinc-500 dark:text-zinc-500">
              +{useCase.features.length - 3}
            </span>
          )}
        </div>

        {/* Read more */}
        <div className="mt-4 pt-4 border-t border-zinc-200/50 dark:border-white/5 flex items-center gap-1.5 text-sm font-medium text-indigo-600 dark:text-indigo-400 opacity-0 group-hover:opacity-100 transition-opacity">
          Read more
          <ArrowRight size={14} weight="bold" className="transition-transform group-hover:translate-x-0.5" />
        </div>
      </div>
    </Link>
  );
}
