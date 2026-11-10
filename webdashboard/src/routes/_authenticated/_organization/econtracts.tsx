import { createFileRoute, redirect } from "@tanstack/react-router";

import { z } from "zod";

import { isFeatureEnabled } from "@/lib/feature-flags";

const contractsSearchSchema = z.object({
  q: z.string().optional(),
});

export const Route = createFileRoute("/_authenticated/_organization/econtracts")({
  beforeLoad: () => {
    if (!isFeatureEnabled("ff_econtracts")) {
      // eslint-disable-next-line @typescript-eslint/only-throw-error
      throw redirect({ to: "/" });
    }
  },
  validateSearch: (search) => contractsSearchSchema.parse(search),
});
