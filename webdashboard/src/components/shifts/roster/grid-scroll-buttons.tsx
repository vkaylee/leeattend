import { ChevronLeft, ChevronRight } from "lucide-react";
import React, { memo } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";

interface GridScrollButtonsProps {
  onScrollLeft: () => void;
  onScrollRight: () => void;
  showLeft: boolean;
  showRight: boolean;
}

export const GridScrollButtons = memo(function GridScrollButtons({
  showLeft,
  showRight,
  onScrollLeft,
  onScrollRight,
}: GridScrollButtonsProps) {
  const { t } = useTranslation();

  return (
    <>
      {showLeft ? (
        <Button
          className="absolute left-4 top-1/2 -translate-y-1/2 z-30 p-3 rounded-full bg-white/90 dark:bg-slate-900/90 backdrop-blur-md border border-slate-250/60 dark:border-slate-800 shadow-premium-lg hover:bg-primary-50 dark:hover:bg-primary-950/40 text-slate-600 dark:text-slate-300 hover:text-primary-600 dark:hover:text-primary-400 hover:scale-105 active:scale-95 transition-all cursor-pointer opacity-0 group-hover/grid:opacity-100 focus:opacity-100"
          onClick={onScrollLeft}
          title={t("common.previous")}
          type="button"
          variant="ghost"
        >
          <ChevronLeft size={16} />
        </Button>
      ) : null}

      {showRight ? (
        <Button
          className="absolute right-4 top-1/2 -translate-y-1/2 z-30 p-3 rounded-full bg-white/90 dark:bg-slate-900/90 backdrop-blur-md border border-slate-250/60 dark:border-slate-800 shadow-premium-lg hover:bg-primary-50 dark:hover:bg-primary-950/40 text-slate-600 dark:text-slate-300 hover:text-primary-600 dark:hover:text-primary-400 hover:scale-105 active:scale-95 transition-all cursor-pointer opacity-0 group-hover/grid:opacity-100 focus:opacity-100"
          onClick={onScrollRight}
          title={t("common.next")}
          type="button"
          variant="ghost"
        >
          <ChevronRight size={16} />
        </Button>
      ) : null}
    </>
  );
});
GridScrollButtons.displayName = "GridScrollButtons";
