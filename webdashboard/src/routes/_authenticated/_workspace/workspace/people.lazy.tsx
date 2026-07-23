import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import MembersPage from "@/pages/members-page";


function MembersPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <MembersPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/people')({
  component: MembersPageWithErrorBoundary,
});
