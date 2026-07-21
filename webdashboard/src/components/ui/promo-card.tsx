import type { LucideIcon } from "lucide-react";

import { ArrowRight } from "lucide-react";
import React, { memo } from "react";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

// ── Component ──────────────────────────────────────────────────────

export interface PromoCardProps {
  /** CTA button text */
  buttonText: string;
  className?: string;
  /** Supporting description */
  description: string;
  /** Eyebrow label above the title */
  eyebrow: string;
  /** CTA link target (render as <a> or pass to Link) */
  href?: string;
  /** Optional React node for the CTA link (overrides default button) */
  linkElement?: React.ReactNode;
  /** onClick handler as alternative to href */
  onClick?: () => void;
  /** Main heading */
  title: string;
  /** Color variant — 'dark' for slate-900, 'primary' for primary-600 */
  variant?: "dark" | "primary";
  /** Lucide icon for the decorative background watermark */
  watermarkIcon: LucideIcon;
}

const PromoCard = memo(function PromoCard({
  eyebrow,
  title,
  description,
  buttonText,
  watermarkIcon: WatermarkIcon,
  variant = "dark",
  linkElement,
  className,
}: PromoCardProps) {
  const isDark = variant === "dark";

  return (
    <div
      className={cn(
        "rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 lg:p-14 text-white relative overflow-hidden group shadow-2xl",
        isDark
          ? "bg-slate-900 shadow-slate-200 dark:shadow-black/40"
          : "bg-primary-600 shadow-primary-900/20",
        className,
      )}
    >
      <div className="relative z-10 h-full flex flex-col items-start">
        <span
          className={cn(
            "text-xs font-bold uppercase tracking-[0.3em] mb-6",
            isDark ? "text-primary-400" : "text-primary-200",
          )}
        >
          {eyebrow}
        </span>
        <Typography className="text-3xl sm:text-5xl font-bold mb-6 sm:mb-8 tracking-tighter leading-[1.1] uppercase" variant="h3">
          {title}
        </Typography>
        <Typography className={cn(
            "mb-8 sm:mb-12 text-base sm:text-lg leading-relaxed max-w-sm font-bold uppercase tracking-tight opacity-80",
            isDark
              ? "text-slate-500 dark:text-slate-400"
              : "text-primary-100",
          )}
          variant="p"
        >
          {description}
        </Typography>
        {linkElement ?? (
          <span
            className={cn(
              "px-8 sm:px-10 py-3 sm:py-4 rounded-2xl font-bold text-xs uppercase tracking-widest transition-all inline-flex items-center gap-4 hover:gap-6 shadow-2xl active:scale-95",
              isDark
                ? "bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-200 hover:shadow-primary-500/20"
                : "bg-slate-900 text-white shadow-black/20",
            )}
          >
            {buttonText} <ArrowRight size={20} />
          </span>
        )}
      </div>
      <WatermarkIcon
        className={cn(
          "absolute -right-20 -bottom-20 w-60 sm:w-80 h-60 sm:h-80 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-1000",
          isDark ? "text-primary-500" : "text-white",
        )}
      />
    </div>
  );
});

export default PromoCard;
