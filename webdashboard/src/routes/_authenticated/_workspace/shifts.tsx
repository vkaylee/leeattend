import { Outlet, createFileRoute } from "@tanstack/react-router";
import React from "react";

import { ShiftsSectionNav } from "@/components/shifts/shifts-section-nav";

export const Route = createFileRoute('/_authenticated/_workspace/shifts')({
  component: () => (
    <div className="space-y-6">
      <ShiftsSectionNav />
      <Outlet />
    </div>
  ),
});
