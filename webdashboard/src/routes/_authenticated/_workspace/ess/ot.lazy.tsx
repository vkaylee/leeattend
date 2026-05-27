import { createLazyFileRoute } from "@tanstack/react-router";

import EssOtPage from "@/pages/ess/ot-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/ess/ot')({
  component: EssOtPage,
});
