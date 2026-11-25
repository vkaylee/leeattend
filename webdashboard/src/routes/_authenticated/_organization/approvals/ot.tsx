import { createFileRoute, redirect } from "@tanstack/react-router";

import { fetchMyPermissions } from "@/lib/api";

const OT_APPROVAL_PERMISSIONS = [
  "overtime:approve",
  "overtime:approve:workflow",
  "overtime:read",
] as const;

export const Route = createFileRoute('/_authenticated/_organization/approvals/ot')({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!OT_APPROVAL_PERMISSIONS.some((p) => permissions.includes(p))) throw redirect({ to: "/" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
});
