import { Outlet, createFileRoute } from "@tanstack/react-router";
import React, { useEffect } from "react";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { ShiftsSectionNav } from "@/components/shifts/shifts-section-nav";
import { useHeaderActions } from "@/hooks/use-header-actions";

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
  component: ShiftsLayout,
});
