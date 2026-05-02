import { createLazyFileRoute } from "@tanstack/react-router";

import AuditLogPage from "@/pages/settings/audit-log-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/audit-logs')({
  component: AuditLogPage,
});
