import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import MyProfilePage from "@/pages/ess/my-profile-page";


function MyProfilePageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <MyProfilePage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_organization/ess/profile')({
  component: MyProfilePageWithErrorBoundary,
});
