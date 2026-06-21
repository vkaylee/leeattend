import { Upload, UserPlus, FileText } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";
import { Link } from "@tanstack/react-router";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";

import type { Employee } from "../../../lib/api/types";

interface EmployeesPageHeaderProps {
  setEditingEmployee: (emp: Employee | null) => void;
  setIsImportDialogOpen: (open: boolean) => void;
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
          <Link to="/econtracts">
            <Button
              icon={<FileText size={18} />}
              variant="outline"
            >
              CONTRACTS HUB
            </Button>
          </Link>
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
            <Button data-testid="add-employee-btn"
              icon={<UserPlus size={18} />}
              onClick={() => {
                setEditingEmployee(null);
                setIsModalOpen(true);
              }}
              variant="ghost"
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
