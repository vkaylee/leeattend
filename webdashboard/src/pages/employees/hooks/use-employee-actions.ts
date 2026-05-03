import { format } from "date-fns";
import { useTranslation } from "react-i18next";

import type { Employee } from "@/lib/api";

interface UseEmployeeActionsProps {
  confirm: (options: any) => Promise<boolean>;
  updateMutation: any;
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
