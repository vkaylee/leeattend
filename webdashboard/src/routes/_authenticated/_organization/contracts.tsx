import { createFileRoute, redirect } from "@tanstack/react-router";
import { z } from "zod";

import { fetchMyPermissions } from "@/lib/api";

const contractsSearchSchema = z.object({
  q: z.string().optional(),
});

export const Route = createFileRoute("/_authenticated/_organization/contracts")({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("documents:contract:read")) throw redirect({ to: "/" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
  validateSearch: (search) => contractsSearchSchema.parse(search),
});
