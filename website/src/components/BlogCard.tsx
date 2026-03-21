import { LocalizedLink as Link } from '@/lib/i18n-routing';
import { useTranslation } from 'react-i18next';
import { Clock, CalendarBlank } from '@phosphor-icons/react';

/* -------------------------------------------------------------------------- */
/*  Types                                                                     */
/* -------------------------------------------------------------------------- */

interface BlogCardProps {
  title: string;
  description: string;
  slug: string;
  date: string;
  readTime: string;
  tags: string[];
  image?: string;
}

/* -------------------------------------------------------------------------- */
/*  BlogCard Component                                                        */
/* -------------------------------------------------------------------------- */

export default function BlogCard({
  title,
  description,
  slug,
  date,
  readTime,
  tags,
  image,
}: BlogCardProps) {
  const { t } = useTranslation();

  return (
    <Link
      to={`/blog/${slug}`}
      className="group block rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl overflow-hidden transition-all duration-300 hover:scale-[1.02] hover:shadow-xl hover:shadow-indigo-500/5 hover:border-indigo-500/20 dark:hover:border-indigo-500/20"
    >
      {/* Image / Gradient Placeholder */}
      <div className="relative h-48 overflow-hidden">
        {image ? (
          <img
            src={image}
            alt={title}
            className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
            loading="lazy"
          />
        ) : (
          <div className="w-full h-full bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-pink-500/20 dark:from-indigo-500/10 dark:via-purple-500/10 dark:to-pink-500/10 flex items-center justify-center">
            <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-indigo-500/30 to-purple-500/30 dark:from-indigo-500/20 dark:to-purple-500/20 backdrop-blur-sm" />
          </div>
        )}

        {/* Gradient overlay at bottom of image */}
        <div className="absolute inset-x-0 bottom-0 h-16 bg-gradient-to-t from-white/80 dark:from-[#030303]/80 to-transparent" />
      </div>

      {/* Content */}
      <div className="p-5 pt-3">
        {/* Tags */}
        {tags.length > 0 && (
          <div className="flex flex-wrap gap-1.5 mb-3">
            {tags.map((tag) => (
              <span
                key={tag}
                className="inline-block px-2.5 py-0.5 text-xs font-medium rounded-full bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/10 dark:border-indigo-500/20"
              >
                {tag}
              </span>
            ))}
          </div>
        )}

        {/* Title */}
        <h3 className="font-display font-semibold text-lg leading-snug text-zinc-900 dark:text-white group-hover:text-indigo-600 dark:group-hover:text-indigo-400 transition-colors line-clamp-2">
          {title}
        </h3>

        {/* Description excerpt */}
        <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed line-clamp-3">
          {description}
        </p>

        {/* Meta: date + read time */}
        <div className="mt-4 flex items-center gap-4 text-xs text-zinc-500 dark:text-zinc-500">
          <span className="flex items-center gap-1">
            <CalendarBlank size={14} weight="duotone" />
            {date}
          </span>
          <span className="flex items-center gap-1">
            <Clock size={14} weight="duotone" />
            {readTime}
          </span>
        </div>
      </div>
    </Link>
  );
}
