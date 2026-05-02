import { createLazyFileRoute } from "@tanstack/react-router";

import WorkspaceBillingPage from "@/pages/workspace/workspace-billing-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/billing')({
  component: WorkspaceBillingPage,
});
