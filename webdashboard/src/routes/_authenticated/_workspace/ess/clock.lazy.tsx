import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import MyClockPage from "@/pages/ess/my-clock-page";


function MyClockPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <MyClockPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_workspace/ess/clock')({
  component: MyClockPageWithErrorBoundary,
});
