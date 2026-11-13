import { createLazyFileRoute, Outlet, useLocation } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import SalaryManagementPage from "@/pages/salary-management-page";

function SalaryManagementPageWithErrorBoundary() {
  const location = useLocation();
  return (
    <ErrorBoundary variant="full">
      {location.pathname === "/salary-management" ? <SalaryManagementPage /> : null}
      <Outlet />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_organization/salary-management")({
  component: SalaryManagementPageWithErrorBoundary,
});
