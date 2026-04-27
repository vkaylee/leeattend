import { createLazyFileRoute } from "@tanstack/react-router";
import PayrollPage from "@/pages/PayrollPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/payroll')({
  component: PayrollPage,
});
