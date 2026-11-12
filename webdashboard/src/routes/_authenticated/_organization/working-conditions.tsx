import { createFileRoute, redirect } from "@tanstack/react-router";
import { z } from "zod";

import { fetchMyPermissions } from "@/lib/api";

const workingConditionsSearchSchema = z.object({
  employeeId: z.string().uuid().optional(),
  q: z.string().optional(),
});

export const Route = createFileRoute("/_authenticated/_organization/working-conditions")({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("employees:read")) throw redirect({ to: "/" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
  validateSearch: (search) => workingConditionsSearchSchema.parse(search),
});
