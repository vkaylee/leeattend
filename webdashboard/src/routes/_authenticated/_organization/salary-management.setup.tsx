import { createFileRoute, redirect } from "@tanstack/react-router";
import { z } from "zod";

import { fetchMyPermissions } from "@/lib/api";

const salarySetupSearchSchema = z.object({
  as_of: z.string().optional(),
  department_id: z.string().optional(),
  employee_id: z.string().min(1).optional(),
  employment_id: z.string().min(1).optional(),
  employer_id: z.string().optional(),
  page: z.coerce.number().int().min(1).optional(),
  search: z.string().optional(),
  status: z.string().optional(),
});

export const Route = createFileRoute("/_authenticated/_organization/salary-management/setup")({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("payroll:manage") && !permissions.includes("payroll:components:manage")) throw redirect({ to: "/" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
  validateSearch: (search) => salarySetupSearchSchema.parse(search),
});
