import { useQuery } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { fetchEmployeePhases, fetchEmploymentPhases } from "@/lib/api/employees";
import { EmployeePhase } from "@/lib/api/types";
import { createQueryKey } from "@/lib/query-keys";
import { useEmployeeContracts } from "@/pages/employees/hooks/use-employee-contracts";

import AssignMilestoneModal from "./assign-milestone-modal";
import PhaseJourneyTimeline from "./phase-journey-timeline";
import UpdatePhaseStatusModal from "./update-phase-status-modal";


interface Props {
  employeeId: string;
  formatDate: (date: string) => string;
}

export default function EmployeePhasesTab({employeeId, formatDate }: Props) {
  const { t } = useTranslation();

  const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);
  const [isStatusModalOpen, setIsStatusModalOpen] = useState(false);
  const [selectedPhase, setSelectedPhase] = useState<EmployeePhase | null>(null);

  const { data: phases, isLoading: isLoadingPhases } = useQuery({
    queryKey: createQueryKey("employee-phases", employeeId),
    queryFn: () => fetchEmployeePhases(employeeId),
    enabled: !!employeeId,
  });

  const { data: definitions } = useQuery({ queryKey: createQueryKey("employment-phases"), queryFn: () => fetchEmploymentPhases(),
 });

  const { contracts } = useEmployeeContracts(employeeId);

  const handleOpenStatusModal = (phase: EmployeePhase) => {
    setSelectedPhase(phase);
    setIsStatusModalOpen(true);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between px-2">
        <div className="max-w-xl">
          <Typography
            className="text-sm font-bold text-slate-800 dark:text-slate-100 uppercase tracking-widest flex items-center gap-2"
            variant="h3"
          >
            {t("employees.phases.journey_title")}
            <span className="bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 px-2 py-0.5 rounded-full text-[10px] normal-case tracking-normal border border-blue-100 dark:border-blue-800/50">
              {t("employees.phases.evaluation_purpose")}
            </span>
          </Typography>
          <Typography
            className="text-[11px] font-medium text-slate-500 dark:text-slate-400 mt-2 leading-relaxed"
            variant="p"
          >
            {t("employees.phases.journey_description")}
          </Typography>
        </div>
        <PermissionGate permission="employment_phases:manage">
          <Button
            className="flex items-center gap-2 bg-primary-50 dark:bg-primary-950/50 text-primary-600 dark:text-primary-400 px-4 py-2 rounded-xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-100 dark:hover:bg-primary-900 transition-all border border-primary-100 dark:border-primary-800 active:scale-95 h-auto shadow-none"
            onClick={() => {
              setIsAssignModalOpen(true);
            }}
            type="button"
            variant="ghost"
          >
            <Plus size={14} /> {t("employees.phases.add_milestone")}
          </Button>
        </PermissionGate>
      </div>

      <div className="space-y-4">
        <PhaseJourneyTimeline
          definitions={definitions}
          formatDate={formatDate}
          isLoading={isLoadingPhases}
          onOpenStatusModal={handleOpenStatusModal}
          phases={phases}
        />
      </div>

      {isAssignModalOpen ? (
        <AssignMilestoneModal contracts={contracts}
        definitions={definitions}
        employeeId={employeeId}
        employmentId={undefined}
        isOpen={isAssignModalOpen}
        onClose={() => {
          setIsAssignModalOpen(false);
        }}  />
      ) : null}

      {isStatusModalOpen && selectedPhase ? (
        <UpdatePhaseStatusModal employeeId={employeeId}
        employmentId={undefined}
        isOpen={isStatusModalOpen}
        onClose={() => {
          setIsStatusModalOpen(false);
          setSelectedPhase(null);
        }}
        selectedPhase={selectedPhase}  />
      ) : null}
    </div>
  );
}

