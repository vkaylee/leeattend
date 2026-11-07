import { Outlet, useLocation } from "@tanstack/react-router";
import React, { useCallback, useEffect, useState } from "react";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { HeaderActionsProvider } from "@/hooks/use-header-actions";
import { useI18nSync } from "@/hooks/use-i18n-sync";
import { useTheme } from "@/hooks/use-theme";
import { useWorkspace } from "@/hooks/use-workspace";
import { cn } from "@/lib/utils";

import { CommandPalette } from "./command-palette";
import { Sidebar } from "./sidebar";
import { Topbar } from "./topbar";

export interface AppShellProps {
  variant?: "workspace" | "infra";
}

// ── AppShell ───────────────────────────────────────────────────────
export default function AppShell({ variant: _variant }: AppShellProps) {
  useTheme();
  useI18nSync();
  const { workspace: currentWorkspace } = useWorkspace();
  const location = useLocation();
  const isOrgChart = location.pathname.endsWith("/org-chart");

  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState<boolean>(() => {
    try {
      return localStorage.getItem("leeattend:sidebar_collapsed") === "true";
    } catch {
      return false;
    }
  });
  const [isMobileSidebarOpen, setIsMobileSidebarOpen] = useState<boolean>(false);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState<boolean>(false);

  const toggleSidebarCollapse = useCallback(() => {
    setIsSidebarCollapsed((prev) => {
      const next = !prev;
      try {
        localStorage.setItem("leeattend:sidebar_collapsed", String(next));
      } catch {
        // ignore storage errors
      }
      return next;
    });
  }, []);

  // Global shortcut: Cmd+K / Ctrl+K opens Command Palette
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setIsCommandPaletteOpen((prev) => !prev);
      }
    };
    const handleOpenEvent = () => setIsCommandPaletteOpen(true);
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("open-command-palette", handleOpenEvent);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("open-command-palette", handleOpenEvent);
    };
   }, []);

  return (
    <HeaderActionsProvider>
      <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground font-sans">
        {/* ── Left Collapsible Navigation Sidebar ── */}
        <Sidebar
          isCollapsed={isSidebarCollapsed}
          isMobileOpen={isMobileSidebarOpen}
          onMobileClose={() => setIsMobileSidebarOpen(false)}
          onToggleCollapse={toggleSidebarCollapse}
        />

        {/* ── Main Application Viewport ── */}
        <div className="flex flex-1 flex-col overflow-hidden min-w-0">
          <Topbar
            onOpenMobileMenu={() => setIsMobileSidebarOpen(true)}
            onOpenSearch={() => setIsCommandPaletteOpen(true)}
            workspaceName={currentWorkspace?.name ?? "Workspace"}
          />

          <main
            className={cn(
              "flex-1 bg-muted/20 min-w-0",
              isOrgChart
                ? "overflow-hidden p-0"
                : "overflow-y-auto p-4 md:p-6 lg:p-8"
            )}
            data-testid="main-content-viewport"
          >
            <ErrorBoundary>
              <Outlet />
            </ErrorBoundary>
          </main>
        </div>

        {/* ── Global Command Palette (⌘K) ── */}
        <CommandPalette
          isOpen={isCommandPaletteOpen}
          onClose={() => setIsCommandPaletteOpen(false)}
        />
      </div>
    </HeaderActionsProvider>
  );
}
