import { Plus, Clock } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";
import { useMyProfile } from "@/hooks/use-ess";
import { useMyOtAssignments, useEmployeeQuota } from "@/hooks/use-overtime-requests";
import { useOrganization } from "@/hooks/use-organization";

import { CreateOtRequestModal } from "./components/create-ot-request-modal";
import { MyOtAssignmentsList } from "./components/my-ot-assignments-list";
import { OtQuotaWidget } from "./components/ot-quota-widget";

export default function EssOtPage() {
  const { t } = useTranslation();
  const { employee } = useMyProfile();
  const employeeId = employee?.id;

  const [createModalOpen, setCreateModalOpen] = useState(false);

  const { data: assignments, isLoading: loadingAssignments } = useMyOtAssignments(employee?.active_employment_id ?? undefined);
  const { data: quota, isLoading: loadingQuota } = useEmployeeQuota(employeeId ?? "", new Date().toISOString().split("T")[0]);

  if (!employeeId) return null;

  return (
    <PageContainer>
      <PageHeader
        actions={
          <Button
            icon={<Plus className="w-4 h-4" />}
            onClick={() => { setCreateModalOpen(true); }}
            variant="primary"
          >
            {t("ess.ot.request", { defaultValue: "Request OT" })}
          </Button>
        }
        data-testid="ess-ot-title"
        icon={<Clock size={24} />}
        subtitle={t("ess.ot.subtitle", { defaultValue: "Request overtime or respond to assignments" })}
        title={t("ess.ot.title", { defaultValue: "My Overtime" })}
      />

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <div className="lg:col-span-2 space-y-8">
          <MyOtAssignmentsList assignments={assignments ?? []} isLoading={loadingAssignments}  />
        </div>
        <div className="space-y-8">
          <OtQuotaWidget isLoading={loadingQuota} quota={quota} />
        </div>
      </div>

      {createModalOpen ? (
        <CreateOtRequestModal employeeId={employeeId}
        isOpen={createModalOpen}
        onClose={() => { setCreateModalOpen(false); }}  />
      ) : null}
    </PageContainer>
  );
}
