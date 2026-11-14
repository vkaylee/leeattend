import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useSearch } from "@tanstack/react-router";
import { Plus, Scale, Settings } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type { EmployeeContract } from "@/lib/api";

import { EmployeeSearch } from "@/components/common/employee-search";
import ContractFormModal from "@/components/features/employees/contract-form-modal";
import { ContractsFilters } from "@/components/features/employees/contracts-filters";
import { ContractsTable } from "@/components/features/employees/contracts-table";
import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";
import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { deleteEmployeeContract, fetchContracts } from "@/lib/api/contracts";
import { isFeatureEnabled } from "@/lib/feature-flags";
import { ROUTES } from "@/lib/routes";
import { createQueryKey } from "@/lib/query-keys";


interface SelectEmployeeModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (employeeId: string) => void;
}

function SelectEmployeeModal({ isOpen, onClose, onSelect }: SelectEmployeeModalProps) {
  const { t } = useTranslation();
  const [employeeId, setEmployeeId] = useState("");

  return (
    <Modal
      icon={<Plus size={24} />}
      isOpen={isOpen}
      onClose={onClose}
      subtitle={t("employees.details.contracts.select_employee_subtitle", { defaultValue: "Select an employee to sign/create a contract" })}
      title={t("employees.details.contracts.select_employee", { defaultValue: "Select Employee" })}
    >
      <div className="space-y-4 pt-4">
        <EmployeeSearch
          data-testid="contracts-employee-search"
          onChange={(value) => { setEmployeeId(typeof value === "string" ? value : value[0] ?? ""); }}
          placeholder={t("employees.table.search_placeholder", { defaultValue: "Search employee by name or code..." })}
          value={employeeId}
        />
        <Button
          className="w-full"
          disabled={!employeeId}
          onClick={() => {
            onSelect(employeeId);
            setEmployeeId("");
            onClose();
          }}
          variant="primary"
        >
          {t("common.save", { defaultValue: "Continue" })}
        </Button>
      </div>
    </Modal>
  );
}

export default function ContractsPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  const { confirm, ConfirmationDialog } = useConfirm();

  const searchParams = useSearch({ from: "/_authenticated/_organization/contracts" });

  const [search, setSearch] = useState(searchParams.q ?? "");
  const [selectedType, setSelectedType] = useState<string>("all");
  const [selectedStatus, setSelectedStatus] = useState<string>("all");

  const [isSelectEmpOpen, setIsSelectEmpOpen] = useState(false);
  const [activeEmployeeId, setActiveEmployeeId] = useState<string | null>(null);
  const [isContractFormOpen, setIsContractFormOpen] = useState(false);
  const [editingContract, setEditingContract] = useState<EmployeeContract | null>(null);

  const { data: contracts = [], isLoading: isLoadingContracts } = useQuery({
    queryKey: createQueryKey("organization-contracts"),
    queryFn: () => fetchContracts(),
  });

  const deleteMutation = useMutation({
    mutationFn: ({ employeeId, contractId }: { employeeId: string; contractId: string }) =>
      deleteEmployeeContract(employeeId, contractId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey("organization-contracts") });
      toast.success(t("common.deleted_success", { defaultValue: "Contract deleted successfully" }));
    },
    onError: (err: unknown) => {
      toast.error(err instanceof Error ? err.message : "Failed to delete contract");
    },
  });

  const handleDelete = async (employeeId: string, contractId: string, contractNumber?: string | null) => {
    const isConfirmed = await confirm({
      title: t("common.delete", { defaultValue: "Delete Contract" }),
      message: t("employees.details.contracts.delete_confirm", {
        defaultValue: `Are you sure you want to delete contract ${contractNumber ?? ""} ? This action cannot be undone.`
      }),
      confirmLabel: t("common.delete", { defaultValue: "Delete" }),
      cancelLabel: t("common.cancel", { defaultValue: "Cancel" }),
    });

    if (isConfirmed) {
      deleteMutation.mutate({ employeeId, contractId });
    }
  };

  const filteredContracts = useMemo(() => {
    return contracts.filter((c) => {
      const nameMatch = c.employee_name.toLowerCase().includes(search.toLowerCase());
      const codeMatch = c.employee_code.toLowerCase().includes(search.toLowerCase());
      const numMatch = c.contract_number?.toLowerCase().includes(search.toLowerCase()) ?? false;
      const matchesSearch = nameMatch || codeMatch || numMatch;
      const matchesType = selectedType === "all" || c.contract_type === selectedType;
      const matchesStatus = selectedStatus === "all" || c.status === selectedStatus;
      return matchesSearch && matchesType && matchesStatus;
    });
  }, [contracts, search, selectedType, selectedStatus]);

  const isPageLoading = isLoadingContracts;

  return (
    <PageContainer>
      <PageHeader
        actions={
          <div className="flex flex-wrap items-center gap-3">
            {isFeatureEnabled("ff_econtracts") ? (
              <Link
                className="flex items-center gap-2 text-xs font-bold text-slate-600 dark:text-slate-400 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl px-4 py-2.5 hover:bg-slate-50 dark:hover:bg-slate-850 hover:text-primary-600 transition-all shadow-sm"
                to={ROUTES.SETTINGS.ECONTRACTS}
              >
                <Settings size={14} /> E-CONTRACT SETTINGS
              </Link>
            ) : null}
            <Button
              className="flex items-center gap-1.5 px-4 py-2.5 rounded-xl text-xs font-bold shadow-sm"
              onClick={() => {
                setEditingContract(null);
                setActiveEmployeeId(null);
                setIsSelectEmpOpen(true);
              }}
              variant="secondary"
            >
              <Plus size={16} /> ADD CONTRACT
            </Button>
          </div>
        }
        icon={<Scale size={24} />}
        subtitle={t("common.contracts_subtitle", { defaultValue: "Manage employment contracts across the organization." })}
        title={t("common.contracts", { defaultValue: "Employment contracts" })}
      />

      <ContractsFilters
        onSearchChange={setSearch}
        onStatusChange={setSelectedStatus}
        onTypeChange={setSelectedType}
        search={search}
        selectedStatus={selectedStatus}
        selectedType={selectedType}
      />

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] shadow-sm overflow-hidden relative min-h-[400px]">
        <ContractsTable
          contracts={filteredContracts}
          isLoading={isPageLoading}
          onClearFilters={() => {
            setSearch("");
            setSelectedType("all");
            setSelectedStatus("all");
          }}
          onDelete={handleDelete}
          onEdit={(contract, employeeId) => {
            setEditingContract(contract);
            setActiveEmployeeId(employeeId);
            setIsContractFormOpen(true);
          }}
          search={search}
          selectedStatus={selectedStatus}
          selectedType={selectedType}
        />
      </div>

      <SelectEmployeeModal
        isOpen={isSelectEmpOpen}
        onClose={() => { setIsSelectEmpOpen(false); }}
        onSelect={(employeeId) => {
          setActiveEmployeeId(employeeId);
          setIsContractFormOpen(true);
        }}
      />

      {isContractFormOpen && activeEmployeeId ? (
        <ContractFormModal editingContract={editingContract}
        employeeId={activeEmployeeId}
        isOpen={isContractFormOpen}
        onClose={() => {
          setIsContractFormOpen(false);
          setActiveEmployeeId(null);
          setEditingContract(null);
        }}  />
      ) : null}

      <ConfirmationDialog />
    </PageContainer>
  );
}
