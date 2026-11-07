import { Link } from "@tanstack/react-router";
import { Upload, UserPlus, FileText } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";

import type { Employee } from "../../../lib/api/types";

interface EmployeesPageHeaderProps {
  setEditingEmployee: (emp: Employee | null) => void;
  setIsImportDialogOpen: (open: boolean) => void;
  setIsModalOpen: (open: boolean) => void;
  setIsTerminationImportOpen: (open: boolean) => void;
}

export const EmployeesPageHeader = ({
  setIsImportDialogOpen,
  setIsTerminationImportOpen,
  setEditingEmployee,
  setIsModalOpen,
}: EmployeesPageHeaderProps) => {
  const { t } = useTranslation();

  return (
    <PageHeader
      actions={
        <div className="flex items-center gap-2.5 flex-wrap sm:flex-nowrap">
          <Link to="/econtracts">
            <Button
              icon={<FileText size={16} />}
              variant="outline"
            >
              {t("common.econtracts", { defaultValue: "Hợp đồng điện tử" })}
            </Button>
          </Link>
          <PermissionGate permission="employees:update">
            <Button
              data-testid="import-termination-btn"
              icon={<Upload size={16} />}
              onClick={() => {
                setIsTerminationImportOpen(true);
              }}
              variant="outline"
            >
              {t("employees.bulk_termination_btn")}
            </Button>
          </PermissionGate>
          <PermissionGate permission="employees:create">
            <Button
              data-testid="import-employee-btn"
              icon={<Upload size={16} />}
              onClick={() => {
                setIsImportDialogOpen(true);
              }}
              variant="outline"
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
                setEditingEmployee(null);
                setIsModalOpen(true);
              }}
              variant="primary"
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
