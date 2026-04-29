import { createLazyFileRoute } from "@tanstack/react-router";

import WorkspaceBillingPage from "@/pages/workspace/WorkspaceBillingPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/billing')({
  component: WorkspaceBillingPage,
});
