import { createLazyFileRoute } from "@tanstack/react-router";
import OtApprovalsPage from "@/pages/approvals/OtApprovalsPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/approvals/ot')({
  component: OtApprovalsPage,
});
