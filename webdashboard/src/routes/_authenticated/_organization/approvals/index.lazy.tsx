import { createLazyFileRoute } from "@tanstack/react-router";

import UnifiedInboxPage from "@/pages/approvals/unified-inbox-page";

export const Route = createLazyFileRoute(
  "/_authenticated/_organization/approvals/"
)({
  component: UnifiedInboxPage,
});
