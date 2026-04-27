import { createFileRoute } from "@tanstack/react-router";
import WorkspaceBillingPage from "@/pages/workspace/WorkspaceBillingPage";

export const Route = createFileRoute(
  "/_authenticated/_workspace/workspace/settings/billing"
)({
  component: WorkspaceBillingPage,
});
