import { Outlet, useLocation } from "@tanstack/react-router";
import { useTheme } from "@/hooks/use-theme";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { HeaderActionsProvider } from "@/hooks/use-header-actions";
import { useI18nSync } from "@/hooks/use-i18n-sync";
import { useNavSidebar } from "@/hooks/use-nav-sidebar";
import { useWorkspace } from "@/hooks/use-workspace";
import { useNotifications } from "@/hooks/use-notifications";
import { cn } from "@/lib/utils";

import BottomDrawer from "./bottom-drawer";
import BottomNav from "./bottom-nav";
import FloatingPanel from "./floating-panel";
import IconRail from "./icon-rail";
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

  const {
    isPinned,
    activeGroup,
    isBottomDrawerOpen,
    togglePin,
    setActiveGroup,
    handleGroupHover,
    handlePanelEnter,
    handlePanelLeave,
    setBottomDrawerOpen,
    pathActiveGroup,
  } = useNavSidebar(location.pathname);

  const isOrgChart = location.pathname.endsWith("/org-chart");

  // Determine if the floating panel should be visible
  const panelOpen = activeGroup !== null && activeGroup !== "dashboard" && activeGroup !== "settings";

  // Handle group click: open panel for most groups, navigate directly for dashboard/settings
  const handleGroupClick = (group: string) => {
    if (group === "dashboard" || group === "settings") return;
    setActiveGroup(group as typeof activeGroup);
  };

  // Handle navigation from floating panel
  const handlePanelNavigate = () => {
    if (!isPinned) {
      setActiveGroup(null);
    }
  };


  return (
    <HeaderActionsProvider>
    <div className="flex min-h-screen bg-background text-foreground font-sans relative overflow-x-hidden">
      {/* Desktop: Icon Rail */}
      <IconRail
        activeGroup={activeGroup ?? pathActiveGroup}
        isPinned={isPinned}
        onClickGroup={handleGroupClick}
        onHoverGroup={handleGroupHover}
        onTogglePin={togglePin}
        workspaceLogoUrl={currentWorkspace?.logo_url ?? undefined}
        workspaceName={currentWorkspace?.name ?? "LeeAttend"}
        workspaceTagline={currentWorkspace?.tagline ?? undefined}
      />

      {/* Desktop: Floating Panel */}
      <FloatingPanel
        group={activeGroup}
        isOpen={panelOpen}
        isPinned={isPinned}
        onClose={() => { setActiveGroup(null); }}
        onMouseEnter={handlePanelEnter}
        onMouseLeave={handlePanelLeave}
        onNavigate={handlePanelNavigate}
        onTogglePin={togglePin}
      />

      {/* Main content area */}
      <div
        className={cn(
          "flex-1 flex flex-col min-w-0 h-dvh overflow-hidden relative transition-[margin-left] duration-300",
          isPinned && panelOpen
            ? "lg:ml-[calc(var(--nav-rail-width)+var(--nav-panel-width))]"
            : "lg:ml-[var(--nav-rail-width)]",
        )}
      >

        <Navbar
          currentWorkspaceName={currentWorkspace?.name ?? "Workspace"}
        />
        <main
          data-app-scroll-container
          className={cn(
            "flex-1 relative scroll-smooth",
            isOrgChart ? "overflow-hidden p-0" : "overflow-y-auto py-4 md:py-6 lg:py-8 pl-safe pr-safe md:px-6 lg:px-8",
            "pb-[var(--bottom-nav-height)] md:pb-[var(--bottom-nav-height)] lg:pb-0", // Bottom nav clearance below desktop
          )}
        >
          <div
            className={cn(
              isOrgChart
                ? "w-full h-full"
                : "max-w-7xl mx-auto",
            )}
          >
            <ErrorBoundary>
              <Outlet />
            </ErrorBoundary>
          </div>
        </main>
      </div>

      {/* Mobile: Bottom Navigation Bar */}
      <BottomNav
        activeGroup={activeGroup ?? pathActiveGroup}
        onMoreClick={() => { setBottomDrawerOpen(true); }}
        unreadCount={unreadCount}
      />

      {/* Mobile: Bottom Sheet Drawer */}
      <BottomDrawer
        isOpen={isBottomDrawerOpen}
        onClose={() => { setBottomDrawerOpen(false); }}
      />

    </div>
    </HeaderActionsProvider>
  );
}
