import { createFileRoute, redirect } from "@tanstack/react-router";
import { z } from "zod";

import { fetchMyPermissions } from "@/lib/api";

const employeeCreateSearchSchema = z.object({
  enrollment_id: z.string().min(1).optional(),
});

export const Route = createFileRoute("/_authenticated/_organization/employees_/new")({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("employees:create")) throw redirect({ to: "/employees" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/employees" });
    }
  },
  validateSearch: (search) => employeeCreateSearchSchema.parse(search),
});
