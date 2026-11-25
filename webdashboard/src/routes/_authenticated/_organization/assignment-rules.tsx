import { Outlet, createFileRoute, redirect } from "@tanstack/react-router";
import React from "react";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { fetchMyPermissions } from "@/lib/api";

function AssignmentRulesLayout() {
  return (
    <ErrorBoundary variant="full">
      <Outlet />
    </ErrorBoundary>
  );
}

export const Route = createFileRoute('/_authenticated/_organization/assignment-rules')({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("assignment_rules:manage")) throw redirect({ to: "/" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
  component: AssignmentRulesLayout,
});
