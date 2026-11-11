import { createFileRoute, redirect } from "@tanstack/react-router";
import { z } from "zod";

const employeeDetailSearchSchema = z.object({
  tab: z.string().optional(),
});

const centralizedTabs = {
  assignments: "/assignments",
  "working-conditions": "/working-conditions",
  lifecycle: "/lifecycle",
} as const;

export const Route = createFileRoute(
  "/_authenticated/_organization/employees_/$employeeId",
)({
  beforeLoad: ({ params, search }) => {
    const tab = (search as { tab?: string }).tab;
    const target = tab && centralizedTabs[tab as keyof typeof centralizedTabs];
    if (target) {
      throw redirect({
        to: target,
        search: { employeeId: params.employeeId } as never,
      });
    }
  },
  validateSearch: (search) => employeeDetailSearchSchema.parse(search),
});
