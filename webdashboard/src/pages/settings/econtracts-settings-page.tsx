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
        icon={<Scale size={20} />}
        subtitle="Manage legal entity e-signing credentials, routing precedence rules, and tracking transactions"
        title="e-Contracts Integration"
      />
      <div className="bg-white dark:bg-slate-900/90 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 p-8 shadow-2xs">
        <EmptyState
          description="The e-signature backend is under active development. Check back soon."
          icon={<Scale size={32} />}
          title="Coming Soon"
        />
      </div>
    </PageContainer>
  );
}
