import { createFileRoute, Link, Outlet, useLocation } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";
import { useTranslation } from "react-i18next";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

export const Route = createFileRoute("/_authenticated/_workspace/workspace/settings")({
  component: SettingsPageLayout,
});

function SettingsPageLayout() {
  const { t } = useTranslation();
  const location = useLocation();
  const isHub = location.pathname === ROUTES.SETTINGS.ROOT;

  return (
    <div className={cn(isHub ? "" : "space-y-6")}>
      {/* Back link — only on sub-pages */}
      {!isHub && (
        <Link
          className="inline-flex items-center gap-2 text-sm font-bold text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
          to={ROUTES.SETTINGS.ROOT}
        >
          <ArrowLeft className="w-4 h-4" />
          {t("common.settings.title")}
        </Link>
      )}

      {/* Content */}
      {isHub ? (
        <ErrorBoundary key={location.pathname}>
          <Outlet />
        </ErrorBoundary>
      ) : (
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500 bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm p-8 lg:p-10 min-h-full">
          <ErrorBoundary key={location.pathname}>
            <Outlet />
          </ErrorBoundary>
        </div>
      )}
    </div>
  );
}
