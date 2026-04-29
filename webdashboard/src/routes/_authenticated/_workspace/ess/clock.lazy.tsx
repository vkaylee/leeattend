import { createLazyFileRoute } from "@tanstack/react-router";

import MyClockPage from "@/pages/ess/MyClockPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/ess/clock')({
  component: MyClockPage,
});
