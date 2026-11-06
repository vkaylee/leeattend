import { forwardRef, memo } from "react";
import { useTranslation } from "react-i18next";

import { motion } from "framer-motion";

import type { FeatureItem } from "@/lib/feature-registry";

import { cn } from "@/lib/utils";

export interface DsmSquircleIconProps {
  isFocused?: boolean;
  item: FeatureItem;
  onClick: (item: FeatureItem) => void;
  size?: "md" | "lg" | "xl";
  testId?: string;
}

export const DsmSquircleIcon = memo(forwardRef<HTMLButtonElement, DsmSquircleIconProps>(
  function DsmSquircleIcon(
    {
      item,
      onClick,
      size = "lg",
      testId,
      isFocused = false,
    },
    ref,
  ) {
    const { t } = useTranslation();
    const Icon = item.icon;

    const label = item.titleKey
      ? t(item.titleKey, { defaultValue: item.titleFallback })
      : item.titleFallback;

    // ponytail: clean 64px max tile with 2-line fixed baseline; upgrade if sublabels or badges are requested
    const sizeClasses = {
      md: {
        box: "w-11 h-11 sm:w-12 sm:h-12 rounded-xl sm:rounded-2xl",
        icon: "w-5 h-5 sm:w-6 sm:h-6",
        text: "text-[11px] sm:text-xs max-w-[80px]",
      },
      lg: {
        box: "w-14 h-14 sm:w-16 sm:h-16 rounded-2xl",
        icon: "w-7 h-7 sm:w-8 sm:h-8",
        text: "text-xs max-w-[92px] sm:max-w-[96px]",
      },
      xl: {
        box: "w-16 h-16 sm:w-18 sm:h-18 rounded-2xl",
        icon: "w-8 h-8 sm:w-9 sm:h-9",
        text: "text-xs max-w-[96px] sm:max-w-[104px]",
      },
    }[size];

    return (
      <motion.button
        ref={ref}
        type="button"
        title={label}
        aria-label={label}
        tabIndex={isFocused ? 0 : -1}
        data-testid={testId ?? item.testId}
        data-focused={isFocused ? "true" : undefined}
        onClick={() => onClick(item)}
        whileHover={{ scale: 1.05, y: -2 }}
        whileTap={{ scale: 0.96 }}
        transition={{ type: "spring", stiffness: 400, damping: 25 }}
        className={cn(
          "group flex flex-col items-center text-center p-2 rounded-2xl cursor-pointer select-none relative",
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900",
          "transition-all duration-150",
          isFocused && "ring-2 ring-primary-400 ring-offset-2 ring-offset-slate-900 scale-105 z-10 shadow-2xl",
        )}
      >
        {/* Clean Squircle Tile (No Skeuomorphic Gloss) */}
        <div
          className={cn(
            "relative flex items-center justify-center shrink-0 bg-gradient-to-br shadow-md group-hover:shadow-xl border border-white/15 dark:border-white/10",
            "transition-all duration-200",
            item.gradient,
            sizeClasses.box,
          )}
        >
          {/* Icon */}
          <Icon
            className={cn(
              "text-white drop-shadow-sm transition-transform duration-200 group-hover:scale-105",
              sizeClasses.icon,
            )}
          />
        </div>
        {/* App Label - Uniform 2-line baseline height */}
        <span
          className={cn(
            "mt-2 text-slate-200 font-medium leading-snug line-clamp-2 min-h-[2rem] flex items-center justify-center text-center",
            "group-hover:text-white transition-colors duration-150",
            sizeClasses.text,
          )}
        >
          {label}
        </span>
      </motion.button>
    );
  },
));
