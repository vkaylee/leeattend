import { Outlet, useLocation } from "@tanstack/react-router";
import { useState } from "react";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { DsmLauncherOverlay, useDsmLauncher } from "@/components/features/dsm-launcher";
import { HeaderActionsProvider } from "@/hooks/use-header-actions";
import { useI18nSync } from "@/hooks/use-i18n-sync";
import { getActiveNavGroup } from "@/hooks/use-nav-menu-items";
import { useNotifications } from "@/hooks/use-notifications";
import { useTheme } from "@/hooks/use-theme";
import { useWorkspace } from "@/hooks/use-workspace";
import { cn } from "@/lib/utils";

import BottomDrawer from "./bottom-drawer";
import BottomNav from "./bottom-nav";
import FloatingDock from "./floating-dock";
import Navbar from "./navbar";

export interface AppShellProps {
  variant?: "workspace" | "infra";
}

// ── AppShell ───────────────────────────────────────────────────────
export default function AppShell({ variant: _variant }: AppShellProps) {
  useTheme();
  useI18nSync();
  const location = useLocation();
  const { workspace: currentWorkspace } = useWorkspace();
  const { unreadCount } = useNotifications();
  const dsmLauncher = useDsmLauncher();
  const [isBottomDrawerOpen, setBottomDrawerOpen] = useState(false);

  const isOrgChart = location.pathname.endsWith("/org-chart");
  const currentNavGroup = getActiveNavGroup(location.pathname);

  return (
    <HeaderActionsProvider>
      <div className="flex min-h-screen flex-col bg-background text-foreground font-sans relative overflow-x-hidden">
        {/* Top Navigation Bar with Breadcrumbs & DSM Search */}
        <Navbar
          currentWorkspaceName={currentWorkspace?.name ?? "Workspace"}
          onOpenLauncher={dsmLauncher.open}
        />

        {/* Main Content Area - Full width with bottom clearance for Floating Dock */}
        <div className="flex-1 flex flex-col min-w-0 h-[calc(100dvh-3.5rem)] overflow-hidden relative">
          <main
            data-app-scroll-container
            className={cn(
              "flex-1 relative scroll-smooth",
              isOrgChart
                ? "overflow-hidden p-0"
                : "overflow-y-auto py-4 md:py-6 lg:py-8 pl-safe pr-safe px-4 md:px-6 lg:px-8",
              // Clearance for Floating OS Dock (desktop) & BottomNav (mobile)
              "pb-24 md:pb-24 lg:pb-28"
            )}
          >
            <div
              className={cn(
                isOrgChart ? "w-full h-full" : "max-w-7xl mx-auto"
              )}
            >
              <ErrorBoundary>
                <Outlet />
              </ErrorBoundary>
            </div>
          </main>
        </div>

        {/* Desktop & Tablet: Modern Floating OS Dock */}
        <FloatingDock onOpenLauncher={dsmLauncher.open} />

        {/* Mobile and tablet: Bottom Navigation Bar (< lg) */}
        <div className="lg:hidden">
          <BottomNav
            activeGroup={currentNavGroup}
            onMoreClick={() => {
              setBottomDrawerOpen(true);
            }}
            unreadCount={unreadCount}
          />
        </div>

        {/* Mobile: Bottom Sheet Drawer */}
        <BottomDrawer
          isOpen={isBottomDrawerOpen}
          onClose={() => {
            setBottomDrawerOpen(false);
          }}
        />

        {/* DSM 7 Main Menu & Feature Launcher Overlay */}
        <DsmLauncherOverlay
          isOpen={dsmLauncher.isOpen}
          onClose={dsmLauncher.close}
        />
      </div>
    </HeaderActionsProvider>
  );
}
