import { createLazyFileRoute } from "@tanstack/react-router";

import DepartmentsPage from "@/pages/departments-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/departments')({
  component: DepartmentsPage,
});
