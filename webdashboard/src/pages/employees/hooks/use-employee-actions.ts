import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import type { Employee } from "@/lib/api";

import { ConfirmOptions } from "@/hooks/use-confirm";

interface UseEmployeeActionsProps {
  confirm: (options: ConfirmOptions) => Promise<boolean>;
  updateMutation: { mutate: (data: { id: string; employee: Partial<Employee> }) => void };
}

export function useEmployeeActions({ confirm, updateMutation }: UseEmployeeActionsProps) {
  const { t } = useTranslation();

  const handleTerminate = async (emp: Employee) => {
    const ok = await confirm({
      message: t("employees.details.terminate_confirm"),
      title: t("common.warning", { defaultValue: "Warning" }),
      variant: "danger"
    });
    
    if (ok) {
      updateMutation.mutate({ 
        id: emp.id, 
        employee: { 
          is_active: false, 
          termination_date: format(new Date(), 'yyyy-MM-dd') 
        } 
      });
    }
  };

  return { handleTerminate };
}
