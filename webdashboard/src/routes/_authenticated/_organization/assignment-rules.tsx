import { Outlet, createFileRoute } from "@tanstack/react-router";
import React from "react";

import { ErrorBoundary } from "@/components/common/error-boundary";

function AssignmentRulesLayout() {
  return (
    <ErrorBoundary variant="full">
      <Outlet />
    </ErrorBoundary>
  );
}

export const Route = createFileRoute('/_authenticated/_organization/assignment-rules')({
  component: AssignmentRulesLayout,
});
