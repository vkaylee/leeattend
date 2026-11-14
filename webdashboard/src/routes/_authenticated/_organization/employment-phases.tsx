import { createFileRoute, redirect } from "@tanstack/react-router";

import { z } from "zod";

import { fetchMyPermissions } from "@/lib/api";

const searchSchema = z.object({
  employeeId: z.uuid().optional(),
  q: z.string().optional(),
  status: z.string().optional(),
  page: z.coerce.number().int().min(1).optional(),
  limit: z.coerce.number().int().min(1).max(3000).optional(),
});

export const Route = createFileRoute("/_authenticated/_organization/employment-phases")({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("employment_phases:read")) throw redirect({ to: "/" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
  validateSearch: (search) => searchSchema.parse(search),
});
