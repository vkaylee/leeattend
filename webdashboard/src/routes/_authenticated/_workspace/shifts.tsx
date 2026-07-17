import { Outlet, createFileRoute } from "@tanstack/react-router";
import React, { useEffect } from "react";

import { ShiftsSectionNav } from "@/components/shifts/shifts-section-nav";
import { useHeaderActions } from "@/hooks/use-header-actions";

export const Route = createFileRoute('/_authenticated/_workspace/shifts')({
  component: () => {
    const { setActions } = useHeaderActions();

    useEffect(() => {
      setActions(<ShiftsSectionNav />);
      return () => setActions(null);
    }, [setActions]);

    return (
      <div className="space-y-6">
        <Outlet />
      </div>
    );
  },
});
