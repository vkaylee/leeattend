import { Outlet, useLocation } from "@tanstack/react-router";
import { Building2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { Typography } from "@/components/ui/typography";
import { HeaderActionsProvider } from "@/hooks/use-header-actions";
import { useI18nSync } from "@/hooks/use-i18n-sync";
import { useNavSidebar } from "@/hooks/use-nav-sidebar";
import { useNotifications } from "@/hooks/use-notifications";
import { useTheme } from "@/hooks/use-theme";
import { useWorkspaces } from "@/hooks/use-workspace";
import { getCurrentWorkspaceId } from "@/lib/api";
import { cn } from "@/lib/utils";

import Button from "../ui/button";
import Modal from "../ui/modal";
import BottomDrawer from "./bottom-drawer";
import BottomNav from "./bottom-nav";
import FloatingPanel from "./floating-panel";
import IconRail from "./icon-rail";
import Navbar from "./navbar";
import WorkspaceSwitcher from "./workspace-switcher";

// ── Types ──────────────────────────────────────────────────────────

export interface AppShellProps {
  variant?: "workspace" | "infra";
}

// ── Workspace Switcher Modal ───────────────────────────────────────

function WorkspaceSwitcherModal({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  const { t } = useTranslation();
  return (
    <Modal
      icon={<Building2 className="w-6 h-6" />}
      isOpen={isOpen}
      maxWidth="lg"
      onClose={onClose}
      subtitle={t("common.office_branch")}
      title={t("common.switch_workspace")}
    >
      <div className="space-y-8">
        <Typography className="text-sm text-slate-500 dark:text-slate-400 font-medium px-1" variant="p">
          {t("common.select_workspace_desc")}
        </Typography>
        <WorkspaceSwitcher />
        <div className="pt-6 border-t border-slate-50 dark:border-slate-800 flex justify-end">
          <Button
            className="px-8 py-3 bg-slate-100 dark:bg-slate-800 text-foreground/70 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-200 dark:hover:bg-slate-700 transition-all"
            onClick={onClose}
            variant="ghost"
          >
            {t("common.close")}
          </Button>
        </div>
      </div>
    </Modal>
  );
}

// ── AppShell ───────────────────────────────────────────────────────

  // eslint-disable-next-line complexity
export default function AppShell({ variant }: AppShellProps) {
  useTheme();
  useI18nSync();
  const location = useLocation();

  const [showSwitcher, setShowSwitcher] = useState(false);

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

  const { workspaces } = useWorkspaces();
  const { unreadCount } = useNotifications();
  const currentWsId = getCurrentWorkspaceId();
  const currentWorkspace = workspaces.find((ws) => ws.id === currentWsId);

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
        onClickWorkspaceSwitcher={() => { setShowSwitcher(true); }}
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
          "md:ml-[var(--nav-rail-width)]", // Always leave room for the icon rail on desktop
        )}
      >
        <Navbar
          currentWorkspaceName={currentWorkspace?.name ?? "Workspace"}
        />

        <main
          data-app-scroll-container
          className={cn(
            "flex-1 relative scroll-smooth",
            isOrgChart ? "overflow-hidden p-0" : "overflow-y-auto p-4 md:p-6 lg:p-8 pl-safe pr-safe",
            "pb-[var(--bottom-nav-height)] md:pb-0", // Bottom nav clearance on mobile
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

      {/* Workspace switcher modal (workspace variant only) */}
      {variant === "workspace" && (
        <WorkspaceSwitcherModal isOpen={showSwitcher} onClose={() => { setShowSwitcher(false); }} />
      )}
    </div>
    </HeaderActionsProvider>
  );
}
