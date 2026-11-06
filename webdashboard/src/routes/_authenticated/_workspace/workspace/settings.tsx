import { createFileRoute, Outlet } from "@tanstack/react-router";
import { Sliders, ChevronDown } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { cn } from "@/lib/utils";
import { SettingsSidebar } from "@/pages/settings/components/settings-sidebar";

export const Route = createFileRoute(
  "/_authenticated/_workspace/workspace/settings"
)({
  component: SettingsPageLayout,
});

function SettingsPageLayout() {
  const { t } = useTranslation();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  return (
    <div className="space-y-4">
      {/* Mobile Selector Bar */}
      <div className="flex md:hidden items-center justify-between p-3 rounded-2xl bg-white dark:bg-slate-900 border border-slate-200/80 dark:border-slate-800/80 shadow-2xs">
        <button
          className="flex items-center justify-between w-full text-xs font-semibold text-slate-700 dark:text-slate-200"
          onClick={() => setMobileMenuOpen((prev) => !prev)}
          type="button"
        >
          <div className="flex items-center gap-2">
            <Sliders className="w-4 h-4 text-primary-600 dark:text-primary-400" />
            <span>{t("common.settings.title", "Cài đặt")}</span>
          </div>
          <span className="flex items-center gap-1 text-[11px] font-bold text-primary-600 dark:text-primary-400">
            {mobileMenuOpen ? t("common.close", "Đóng") : t("settings.categories_menu", "Danh mục")}
            <ChevronDown className={cn("w-3.5 h-3.5 transition-transform", mobileMenuOpen && "rotate-180")} />
          </span>
        </button>
      </div>

      <div className="flex flex-col md:flex-row gap-6 min-h-[calc(100vh-10rem)]">
        {/* Left Column: macOS System Settings Style Master Sidebar */}
        <div className={cn("w-full md:w-72 lg:w-80 shrink-0", !mobileMenuOpen && "hidden md:block")}>
          <SettingsSidebar />
        </div>

        {/* Right Column: Active Setting Detail Pane */}
        <div className={cn("flex-1 min-w-0 bg-white dark:bg-slate-900 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 shadow-xs p-6 md:p-8 lg:p-10", mobileMenuOpen && "hidden md:block")}>
          <ErrorBoundary>
            <Outlet />
          </ErrorBoundary>
        </div>
      </div>
    </div>
  );
}
