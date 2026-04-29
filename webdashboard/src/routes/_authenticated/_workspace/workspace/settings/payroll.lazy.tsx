import { createLazyFileRoute } from "@tanstack/react-router";

import PayrollSettings from "@/pages/settings/PayrollSettings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/payroll')({
  component: PayrollSettings,
});
