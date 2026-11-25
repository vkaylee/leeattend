import { Outlet, createFileRoute, redirect } from "@tanstack/react-router";
import React, { useEffect } from "react";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { ShiftsSectionNav } from "@/components/shifts/shifts-section-nav";
import { useHeaderActions } from "@/hooks/use-header-actions";
import { fetchMyPermissions } from "@/lib/api";

const SHIFTS_ROUTE_PERMISSIONS = [
  "shifts:read",
  "rosters:read",
  "working_patterns:read",
  "schedules:publish:rotating",
  "schedules:amend",
] as const;

function ShiftsLayout() {
  const { setActions } = useHeaderActions();

  useEffect(() => {
    setActions(<ShiftsSectionNav />);
    return () => { setActions(null); };
  }, [setActions]);

  return (
    <div className="space-y-6">
      <ErrorBoundary variant="full">
        <Outlet />
      </ErrorBoundary>
    </div>
  );
}

export const Route = createFileRoute('/_authenticated/_organization/shifts')({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!SHIFTS_ROUTE_PERMISSIONS.some((p) => permissions.includes(p))) {
        throw redirect({ to: "/" });
      }
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
  component: ShiftsLayout,
});
