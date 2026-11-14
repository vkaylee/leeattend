import { createFileRoute, redirect } from "@tanstack/react-router";

import { z } from "zod";

import { fetchMyPermissions } from "@/lib/api";

const salaryEditSearchSchema = z.object({
  as_of: z.string().optional(),
  department_id: z.string().optional(),
  employee_id: z.string().min(1),
  employer_id: z.string().optional(),
  page: z.coerce.number().int().min(1).optional(),
  search: z.string().optional(),
  status: z.string().optional(),
});

export const Route = createFileRoute("/_authenticated/_organization/salary-management/$employmentId/edit")({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("payroll:manage") && !permissions.includes("payroll:components:manage")) throw redirect({ to: "/" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
  validateSearch: (search) => salaryEditSearchSchema.parse(search),
});
