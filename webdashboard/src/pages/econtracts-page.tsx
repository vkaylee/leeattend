import { FileSignature } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import EmptyState from "@/components/ui/empty-state";
import { PageContainer } from "@/components/ui/page-container";
import { PageHeader } from "@/components/ui/page-header";

export default function EContractsPlaceholderPage() {
  const { t } = useTranslation();

  return (
    <PageContainer>
      <PageHeader
        icon={<FileSignature size={24} />}
        subtitle={t("econtracts.not_ready_subtitle", { defaultValue: "Electronic-signature envelopes will appear here when the signing backend is enabled." })}
        title={t("common.econtracts", { defaultValue: "Electronic Contracts" })}
      />
      <EmptyState
        description={t("econtracts.not_ready_description", { defaultValue: "Electronic-signature provider integration is not available yet. Ordinary legal contracts remain available in Contracts." })}
        icon={<FileSignature size={40} />}
        title={t("econtracts.not_ready_title", { defaultValue: "E-signature workflow unavailable" })}
      />
    </PageContainer>
  );
}
