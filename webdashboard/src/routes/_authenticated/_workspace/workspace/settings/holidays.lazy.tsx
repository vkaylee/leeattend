import { createLazyFileRoute } from "@tanstack/react-router";

import HolidaysPage from "@/pages/settings/HolidaysPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/holidays')({
  component: HolidaysPage,
});
