import { createFileRoute } from "@tanstack/react-router";
import AuditLogPage from "@/pages/settings/AuditLogPage";

export const Route = createFileRoute("/_authenticated/_workspace/workspace/settings/audit-logs")({
  component: AuditLogPage,
});
