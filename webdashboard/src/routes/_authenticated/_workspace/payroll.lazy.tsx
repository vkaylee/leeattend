import { createLazyFileRoute } from "@tanstack/react-router";

import PayrollPage from "@/pages/payroll-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/payroll')({
  component: PayrollPage,
});
