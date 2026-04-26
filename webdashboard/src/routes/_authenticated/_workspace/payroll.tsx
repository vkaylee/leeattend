import { createFileRoute } from "@tanstack/react-router";
import PayrollPage from "@/pages/PayrollPage";

export const Route = createFileRoute("/_authenticated/_workspace/payroll")({
  component: PayrollPage,
});
