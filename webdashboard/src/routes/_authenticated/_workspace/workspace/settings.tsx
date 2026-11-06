import { createFileRoute, Outlet } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { SettingsSidebar } from "@/pages/settings/components/settings-sidebar";

export const Route = createFileRoute(
  "/_authenticated/_workspace/workspace/settings"
)({
  component: SettingsPageLayout,
});

function SettingsPageLayout() {
  return (
    <div className="flex flex-col md:flex-row gap-6 min-h-[calc(100vh-10rem)]">
      {/* Left Column: macOS System Settings Style Master Sidebar */}
      <SettingsSidebar className="w-full md:w-72 lg:w-80 shrink-0" />

      {/* Right Column: Active Setting Detail Pane */}
      <div className="flex-1 min-w-0 bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-xs p-6 md:p-8 lg:p-10">
        <ErrorBoundary>
          <Outlet />
        </ErrorBoundary>
      </div>
    </div>
  );
}
