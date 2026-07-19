import { Outlet, createFileRoute } from "@tanstack/react-router";
import React, { useEffect } from "react";

import { ShiftsSectionNav } from "@/components/shifts/shifts-section-nav";
import { useHeaderActions } from "@/hooks/use-header-actions";

export const Route = createFileRoute('/_authenticated/_workspace/shifts')({
  component: () => {
  // eslint-disable-next-line react-hooks/rules-of-hooks
    const { setActions } = useHeaderActions();

  // eslint-disable-next-line react-hooks/rules-of-hooks
    useEffect(() => {
      setActions(<ShiftsSectionNav />);
      return () => { setActions(null); };
    }, [setActions]);

    return (
      <div className="space-y-6">
        <Outlet />
      </div>
    );
  },
});
