import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import ProfilePage from "@/pages/profile-page";


function ProfilePageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <ProfilePage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_organization/profile')({
  component: ProfilePageWithErrorBoundary,
});
