import { createFileRoute } from "@tanstack/react-router";
import PayrollSettings from "@/pages/settings/PayrollSettings";

export const Route = createFileRoute("/_authenticated/_workspace/workspace/settings/payroll")({
  component: PayrollSettings,
});
