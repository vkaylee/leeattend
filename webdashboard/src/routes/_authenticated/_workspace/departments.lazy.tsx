import { createLazyFileRoute } from "@tanstack/react-router";
import DepartmentsPage from "@/pages/DepartmentsPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/departments')({
  component: DepartmentsPage,
});
