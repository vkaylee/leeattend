import { memo } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";

import type { FeatureItem } from "@/lib/feature-registry";
import { cn } from "@/lib/utils";
interface DsmSquircleIconProps {
  item: FeatureItem;
  onClick: (item: FeatureItem) => void;
  size?: "md" | "lg" | "xl";
  testId?: string;
}
export const DsmSquircleIcon = memo(function DsmSquircleIcon({
  item,
  onClick,
  size = "lg",
  testId,
}: DsmSquircleIconProps) {
  const { t } = useTranslation();
  const Icon = item.icon;

  const label = item.titleKey
    ? t(item.titleKey, { defaultValue: item.titleFallback })
    : item.titleFallback;

  const sizeClasses = {
    md: { box: "w-12 h-12 sm:w-14 sm:h-14", icon: "w-6 h-6 sm:w-7 sm:h-7", text: "text-[11px] sm:text-xs max-w-[88px]" },
    lg: { box: "w-16 h-16 sm:w-20 sm:h-20", icon: "w-8 h-8 sm:w-10 sm:h-10", text: "text-xs sm:text-sm max-w-[100px]" },
    xl: { box: "w-20 h-20 sm:w-24 sm:h-24", icon: "w-10 h-10 sm:w-12 sm:h-12", text: "text-xs sm:text-sm max-w-[110px] sm:max-w-[130px]" },
  }[size];

  return (
    <motion.button
      type="button"
      role="button"
      aria-label={label}
      data-testid={testId ?? item.testId}
      onClick={() => onClick(item)}
      whileHover={{ scale: 1.06, y: -3 }}
      whileTap={{ scale: 0.95 }}
      transition={{ type: "spring", stiffness: 400, damping: 25 }}
      className={cn(
        "group flex flex-col items-center text-center p-2 rounded-2xl cursor-pointer select-none",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900",
        "transition-colors duration-150",
      )}
    >
      {/* DSM 7 Squircle Tile */}
      <div
        className={cn(
          "relative flex items-center justify-center shrink-0 rounded-2xl sm:rounded-3xl bg-gradient-to-br shadow-xl border border-white/20 dark:border-white/10",
          "transition-all duration-300 group-hover:shadow-2xl",
          item.gradient,
          sizeClasses.box,
        )}
      >
        {/* Gloss / Sheen Highlight on Top Half */}
        <div className="absolute inset-x-0 top-0 h-1/2 bg-gradient-to-b from-white/25 to-transparent rounded-t-2xl sm:rounded-t-3xl pointer-events-none" />

        {/* Icon */}
        <Icon
          className={cn(
            "text-white drop-shadow-md transition-transform duration-300 group-hover:scale-110",
            sizeClasses.icon,
          )}
        />
      </div>

      {/* App Label */}
      <span
        className={cn(
          "mt-2 text-slate-100 font-medium leading-snug line-clamp-2 drop-shadow-[0_1px_2px_rgba(0,0,0,0.8)]",
          "group-hover:text-white transition-colors duration-200",
          sizeClasses.text,
        )}
      >
        {label}
      </span>
    </motion.button>
  );
});
