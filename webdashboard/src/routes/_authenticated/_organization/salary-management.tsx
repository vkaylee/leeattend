import { createFileRoute, redirect } from "@tanstack/react-router";
import { z } from "zod";

import { fetchMyPermissions } from "@/lib/api";

const salaryManagementSearchSchema = z.object({
  as_of: z.string().optional(),
  department_id: z.string().optional(),
  employer_id: z.string().optional(),
  page: z.coerce.number().int().min(1).optional(),
  search: z.string().optional(),
  status: z.string().optional(),
  unconfigured: z.coerce.boolean().optional(),
});

export const Route = createFileRoute("/_authenticated/_organization/salary-management")({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("payroll:read")) throw redirect({ to: "/" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
  validateSearch: (search) => salaryManagementSearchSchema.parse(search),
});
