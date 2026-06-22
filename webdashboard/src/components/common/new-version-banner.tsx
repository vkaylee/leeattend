import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";

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
      className={`fixed top-0 left-0 right-0 z-[9999] transition-transform duration-300 ${
        leaving ? "-translate-y-full" : "translate-y-0"
      }`}
    >
      <div className="mx-auto max-w-7xl px-4 py-2.5">
        <div className="flex items-center justify-between gap-4 rounded-xl bg-blue-600 px-5 py-3 text-white shadow-lg">
          <div className="flex items-center gap-2 text-sm font-medium">
            <svg className="h-4 w-4 shrink-0" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
              <path d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
            </svg>
            <span>{t("common.new_version_banner.title")}</span>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            <Button className="rounded-lg bg-white/20 px-4 py-1.5 text-sm font-medium hover:bg-white/30 transition-colors"
              onClick={handleRefresh}
              variant="ghost"
            >
              {t("common.new_version_banner.refresh")}
            </Button>
            <Button aria-label={t("common.new_version_banner.dismiss")}
              className="rounded-lg p-1.5 hover:bg-white/20 transition-colors"
              onClick={handleDismiss}
              variant="ghost"
            >
              <svg className="h-4 w-4" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
