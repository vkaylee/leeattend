import { createFileRoute, Outlet } from "@tanstack/react-router";

import { z } from "zod";

import { ErrorBoundary } from "@/components/common/error-boundary";

const workflowsSearchSchema = z.object({
  tab: z.enum(["overtime", "leave", "attendance"]).optional(),
});

function WorkflowsLayout() {
  return (
    <ErrorBoundary variant="full">
      <Outlet />
    </ErrorBoundary>
  );
}

export const Route = createFileRoute(
  '/_authenticated/_organization/settings/workflows',
)({
  validateSearch: workflowsSearchSchema,
  component: WorkflowsLayout,
});


