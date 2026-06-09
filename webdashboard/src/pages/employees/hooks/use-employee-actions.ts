import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import type { Employee } from "@/lib/api";

import { ConfirmOptions } from "@/hooks/use-confirm";

interface UseEmployeeActionsProps {
  confirm: (options: ConfirmOptions) => Promise<boolean>;
  terminateMutation: { mutate: (data: { employeeId: string; employmentId: string; endDate: string }) => void };
}

export function useEmployeeActions({ confirm, terminateMutation }: UseEmployeeActionsProps) {
  const { t } = useTranslation();

  const handleTerminate = async (emp: Employee) => {
    if (!emp.active_employment_id) return;
    const ok = await confirm({
      message: t("employees.details.terminate_confirm"),
      title: t("common.warning", { defaultValue: "Warning" }),
      variant: "danger"
    });
    
    if (ok) {
      terminateMutation.mutate({ 
        employeeId: emp.id, 
        employmentId: emp.active_employment_id,
        endDate: format(new Date(), 'yyyy-MM-dd') 
      });
    }
  };

  return { handleTerminate };
}
