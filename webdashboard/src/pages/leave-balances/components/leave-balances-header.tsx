import { PlayCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { PageHeader } from "@/components/ui/page-header";
interface LeaveBalancesHeaderProps {
  isPending: boolean;
  onRunAccrual: () => void;
  selectedYear: number;
}

export const LeaveBalancesHeader: React.FC<LeaveBalancesHeaderProps> = ({
  isPending,
  onRunAccrual,
  selectedYear,
}) => {
  const { t } = useTranslation();
  const currentYear = new Date().getFullYear();

  return (
    <PageHeader
      actions={
        <Button
          className="w-full md:w-auto shadow-sm h-10 px-5 rounded-xl font-bold"
          disabled={isPending || selectedYear !== currentYear}
          onClick={onRunAccrual}
          size="md"
          title={selectedYear !== currentYear ? t("leave_balances.accrual_current_year_only") : undefined}
          variant="primary"
        >
          <PlayCircle size={18} /> 
          <span className="whitespace-nowrap">
            {isPending ? t("common.processing") : t("leave_balances.run_accrual")}
          </span>
        </Button>
      }
      className="px-2"
      subtitle={t("leave_balances.subtitle")}
      title={t("leave_balances.title")}
    />
  );
};
