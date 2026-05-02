import { createLazyFileRoute } from "@tanstack/react-router";

import OtApprovalsPage from "@/pages/approvals/ot-approvals-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/approvals/ot')({
  component: OtApprovalsPage,
});
