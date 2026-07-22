import { Sparkles, RefreshCw, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface NewVersionBannerProps {
  onDismiss: () => void;
  onRefresh: () => void;
}

export function NewVersionBanner({ onRefresh, onDismiss }: NewVersionBannerProps) {
  const { t } = useTranslation();
  const [leaving, setLeaving] = useState(false);

  const handleRefresh = () => {
    setLeaving(true);
    setTimeout(() => { onRefresh(); }, 300);
  };

  const handleDismiss = () => {
    setLeaving(true);
    setTimeout(() => { onDismiss(); }, 300);
  };

  return (
    <div
      className={`fixed top-0 left-0 right-0 z-[9999] flex justify-center px-4 pt-4 transition-all duration-300 ease-out ${
        leaving ? "-translate-y-full opacity-0" : "translate-y-0 opacity-100"
      }`}
    >
      <div className="pointer-events-auto flex w-full max-w-md items-center gap-4 rounded-[24px] border border-primary-100 bg-white/90 p-3 pl-4 shadow-xl shadow-slate-200/60 backdrop-blur-xl dark:border-primary-900 dark:bg-slate-900/90 dark:shadow-black/40">
        <div className="relative flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-primary-50 dark:bg-primary-950">
          <Sparkles className="h-5 w-5 text-primary-600 dark:text-primary-400" />
          <span className="absolute right-0 top-0 h-2.5 w-2.5 -translate-y-0.5 translate-x-0.5 rounded-full bg-primary-500 ring-2 ring-white dark:ring-slate-900">
            <span className="absolute inset-0 animate-ping rounded-full bg-primary-500 opacity-75" />
          </span>
        </div>

        <div className="flex-1 min-w-0">
          <Typography className="truncate text-sm font-bold text-slate-800 dark:text-slate-100" variant="p">
            {t("common.new_version_banner.title")}
          </Typography>
          <Typography className="truncate text-xs font-medium text-slate-500 dark:text-slate-400" variant="p">
            {t("common.new_version_banner.subtitle")}
          </Typography>
        </div>

        <Button
          className="inline-flex shrink-0 items-center gap-1.5 rounded-2xl bg-primary-600 px-4 py-2.5 text-xs font-bold uppercase tracking-wider text-white shadow-lg shadow-primary-200 transition-all hover:bg-primary-700 active:scale-95 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 dark:shadow-black/30"
          onClick={handleRefresh}
          type="button"
        >
          <RefreshCw className="h-3.5 w-3.5" />
          {t("common.new_version_banner.refresh")}
        </Button>

        <Button
          aria-label={t("common.new_version_banner.dismiss")}
          className="inline-flex shrink-0 items-center justify-center rounded-xl p-2 text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-600 active:scale-95 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 dark:text-slate-500 dark:hover:bg-slate-800 dark:hover:text-slate-300"
          onClick={handleDismiss}
          type="button"
        >
          <X className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
