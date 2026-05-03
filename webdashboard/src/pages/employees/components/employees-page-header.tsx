import { Upload, UserPlus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";

import type { Employee } from "../../../lib/api/types";

interface EmployeesPageHeaderProps {
  setIsImportDialogOpen: (open: boolean) => void;
  setEditingEmployee: (emp: Employee | null) => void;
  setIsModalOpen: (open: boolean) => void;
}

export const EmployeesPageHeader = ({
  setIsImportDialogOpen,
  setEditingEmployee,
  setIsModalOpen,
}: EmployeesPageHeaderProps) => {
  const { t } = useTranslation();

  return (
    <PageHeader
      actions={
        <>
          <PermissionGate permission="employees:create">
            <Button
              data-testid="import-employee-btn"
              icon={<Upload size={18} />}
              onClick={() => {
                setIsImportDialogOpen(true);
              }}
              variant="outline"
            >
              {t("employees.bulk_import_btn").toUpperCase()}
            </Button>
          </PermissionGate>
          <PermissionGate permission="employees:create">
            <Button
              data-testid="add-employee-btn"
              icon={<UserPlus size={18} />}
              onClick={() => {
                setEditingEmployee(null);
                setIsModalOpen(true);
              }}
            >
              {t("employees.add_btn").toUpperCase()}
            </Button>
          </PermissionGate>
        </>
      }
      data-testid="employees-title"
      subtitle={t("employees.subtitle")}
      title={t("employees.title")}
    />
  );
};
