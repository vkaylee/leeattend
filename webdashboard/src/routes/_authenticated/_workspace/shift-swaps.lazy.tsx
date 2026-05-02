import { createLazyFileRoute } from "@tanstack/react-router";

import ShiftSwapsPage from "@/pages/shift-swaps-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/shift-swaps')({
  component: ShiftSwapsPage,
});
