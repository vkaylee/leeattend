import { createFileRoute } from "@tanstack/react-router";
import OtApprovalsPage from "@/pages/approvals/OtApprovalsPage";

export const Route = createFileRoute("/_authenticated/_workspace/approvals/ot")({
  component: OtApprovalsPage,
});
