import { createLazyFileRoute } from "@tanstack/react-router";
import ShiftSwapsPage from "@/pages/ShiftSwapsPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/shift-swaps')({
  component: ShiftSwapsPage,
});
