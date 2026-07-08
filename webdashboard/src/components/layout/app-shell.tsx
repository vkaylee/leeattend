import { Outlet, useLocation } from "@tanstack/react-router";
import { Building2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { Typography } from "@/components/ui/typography";
import { HeaderActionsProvider } from "@/hooks/use-header-actions";
import { useI18nSync } from "@/hooks/use-i18n-sync";
import { useNavSidebar } from "@/hooks/use-nav-sidebar";
import { useTheme } from "@/hooks/use-theme";
import { useWorkspaces } from "@/hooks/use-workspace";
import { getCurrentWorkspaceId } from "@/lib/api";
import { cn } from "@/lib/utils";

import Button from "../ui/button";
import Modal from "../ui/modal";
import FloatingPanel from "./floating-panel";
import IconRail from "./icon-rail";
import Navbar from "./navbar";
import Sidebar from "./sidebar";
import WorkspaceSwitcher from "./workspace-switcher";

// ── Types ──────────────────────────────────────────────────────────

export interface AppShellProps {
  variant?: "workspace" | "infra";
}

// ── Mobile Sidebar Overlay ─────────────────────────────────────────

function SidebarOverlay({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  if (!isOpen) return null;
  return (
    <div
      className="fixed inset-0 bg-slate-900/50 dark:bg-slate-950/60 backdrop-blur-sm z-40 lg:hidden transition-opacity duration-500"
      onClick={onClose}
      onKeyDown={(e) => {
        if (e.key === "Escape" || e.key === "Enter" || e.key === " ") onClose();
      }}
      role="presentation"
    />
  );
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

export default function AppShell({ variant }: AppShellProps) {
  useTheme();
  useI18nSync();
  const location = useLocation();

  const [showSwitcher, setShowSwitcher] = useState(false);

  const {
    isPinned,
    activeGroup,
    isMobileOpen,
    togglePin,
    setActiveGroup,
    handleGroupHover,
    handlePanelEnter,
    handlePanelLeave,
    setMobileOpen,
    pathActiveGroup,
  } = useNavSidebar(location.pathname);

  const { workspaces } = useWorkspaces();
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
    // Also close mobile sidebar if open
    setMobileOpen(false);
  };


  return (
    <HeaderActionsProvider>
    <div className="flex min-h-screen bg-background text-foreground font-sans relative overflow-x-hidden">
      {/* Mobile overlay */}
      <SidebarOverlay isOpen={isMobileOpen} onClose={() => { setMobileOpen(false); }} />

      {/* Mobile sidebar — wrapped in lg:hidden so only visible on small screens */}
      <div className="lg:hidden">
        <Sidebar
          isCollapsed={false}
          isOpen={isMobileOpen}
          onClose={() => { setMobileOpen(false); }}
          openSections={{ org: false, time: false, shifts: false, payroll: false, system: false, ess: false }}
          toggleSection={() => {}}
        />
      </div>

      {/* Desktop: Icon Rail */}
      <IconRail
        activeGroup={activeGroup ?? pathActiveGroup}
        isPinned={isPinned}
        onClickGroup={handleGroupClick}
        onHoverGroup={handleGroupHover}
        onClickWorkspaceSwitcher={() => { setShowSwitcher(true); }}
        onTogglePin={togglePin}
        workspaceLogoUrl={currentWorkspace?.logo_url ?? undefined}
        workspaceName={currentWorkspace?.name ?? "LeeAttend"}
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
          "flex-1 flex flex-col min-w-0 h-screen overflow-hidden relative transition-all duration-300",
          "md:ml-14", // Always leave room for the 56px icon rail on desktop
        )}
      >
        <Navbar
          currentWorkspaceName={currentWorkspace?.name ?? "Workspace"}
          onOpenSidebar={() => { setMobileOpen(true); }}
        />

        <main
          className={cn(
            "flex-1 relative scroll-smooth",
            isOrgChart ? "overflow-hidden p-0" : "overflow-y-auto p-4 md:p-6 lg:p-8",
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

      {/* Workspace switcher modal (workspace variant only) */}
      {variant === "workspace" && (
        <WorkspaceSwitcherModal isOpen={showSwitcher} onClose={() => { setShowSwitcher(false); }} />
      )}
    </div>
    </HeaderActionsProvider>
  );
}
