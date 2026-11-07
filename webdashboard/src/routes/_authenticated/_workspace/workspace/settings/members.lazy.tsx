import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import MembersPage from "@/pages/members-page";

function MembersSettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <MembersPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute(
  "/_authenticated/_workspace/workspace/settings/members",
)({
  component: MembersSettingsWithErrorBoundary,
});
