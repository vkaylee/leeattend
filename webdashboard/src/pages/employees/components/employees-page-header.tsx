import { Link, useNavigate } from "@tanstack/react-router";
import { Upload, UserPlus, FileText } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import { PageHeader } from "@/components/ui/page-header";

import type { EmployeeModalState } from "../types/employee-modals";

interface EmployeesPageHeaderProps {
  openModal?: (modal: EmployeeModalState) => void;
  setIsImportDialogOpen?: (open: boolean) => void;
  setIsTerminationImportOpen?: (open: boolean) => void;
}

export const EmployeesPageHeader = ({
  openModal,
  setIsImportDialogOpen,
  setIsTerminationImportOpen,
}: EmployeesPageHeaderProps) => {
  const { t } = useTranslation();
  const navigate = useNavigate();

  return (
    <PageHeader
      actions={
        <div className="flex items-center gap-2.5 flex-wrap sm:flex-nowrap">
          <Link to="/contracts">
            <Button
              icon={<FileText size={16} />}
              variant="outline"
            >
              {t("common.contracts", { defaultValue: "Hợp đồng lao động" })}
            </Button>
          </Link>
          <PermissionGate permission="employees:update">
            <Button
              data-testid="import-termination-btn"
              icon={<Upload size={16} />}
              onClick={() => {
                if (openModal) openModal({ type: "import-termination" });
                else setIsTerminationImportOpen?.(true);
              }}
            >
              {t("employees.bulk_termination_btn")}
            </Button>
          </PermissionGate>
          <PermissionGate permission="employees:create">
            <Button
              data-testid="import-employee-btn"
              icon={<Upload size={16} />}
              onClick={() => {
                if (openModal) openModal({ type: "import" });
                else setIsImportDialogOpen?.(true);
              }}
            >
              {t("employees.bulk_import_btn")}
            </Button>
          </PermissionGate>
          <PermissionGate permission="employees:create">
            <Button
              className="shrink-0 font-bold shadow-sm"
              data-testid="add-employee-btn"
              icon={<UserPlus size={16} />}
              onClick={() => {
                void navigate({ to: "/employees/new" });
              }}
            >
              {t("employees.add_btn")}
            </Button>
          </PermissionGate>
        </div>
      }
      data-testid="employees-title"
      subtitle={t("employees.subtitle")}
      title={t("employees.title")}
    />
  );
};
