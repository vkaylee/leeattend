import { createFileRoute } from "@tanstack/react-router";
import EmployeesPage from "@/pages/EmployeesPage";

export const Route = createFileRoute("/_authenticated/_workspace/employees")({
  component: EmployeesPage,
});
