import { createLazyFileRoute } from "@tanstack/react-router";

import EmployeesPage from "@/pages/employees-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/employees')({
  component: EmployeesPage,
});
