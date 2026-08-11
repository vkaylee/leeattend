import { Scale } from "lucide-react";
import React from "react";

import EmptyState from "@/components/ui/empty-state";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";

/** e-Contracts e-signature integration — feature gate placeholder.
 *  Backend HTTP endpoints are pending wiring
 *  (@see apiserver/src/services/econtract/ for provider trait + dispatcher).
 */
export default function EContractsSettingsPage() {
  return (
    <PageContainer>
      <PageHeader
        icon={<Scale size={24} />}
        subtitle="Manage legal entity e-signing credentials, routing precedence rules, and tracking transactions"
        title="e-Contracts Integration"
      />
      <EmptyState
        description="The e-signature backend is under active development. Check back soon."
        icon={<Scale size={36} />}
        title="Coming Soon"
      />
    </PageContainer>
  );
}
