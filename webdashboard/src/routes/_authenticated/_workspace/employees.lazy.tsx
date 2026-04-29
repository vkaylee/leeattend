import { createLazyFileRoute } from "@tanstack/react-router";

import EmployeesPage from "@/pages/EmployeesPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/employees')({
  component: EmployeesPage,
});
