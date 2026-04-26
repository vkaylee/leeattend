import { createFileRoute } from "@tanstack/react-router";
import ShiftSwapsPage from "@/pages/ShiftSwapsPage";

export const Route = createFileRoute("/_authenticated/_workspace/shift-swaps")({
  component: ShiftSwapsPage,
});
