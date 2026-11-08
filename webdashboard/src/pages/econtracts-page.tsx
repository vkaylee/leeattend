import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useSearch } from "@tanstack/react-router";
import { Plus, Scale, Settings } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type { EmployeeContract } from "@/lib/api";
import type { Employee } from "@/lib/api/types";

import ContractFormModal from "@/components/features/employees/contract-form-modal";
import { EContractsFilters } from "@/components/features/employees/econtracts-filters";
import { EContractsTable } from "@/components/features/employees/econtracts-table";
import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import { PageContainer } from "@/components/ui/page-container";
import Modal from "@/components/ui/modal";
import PageHeader from "@/components/ui/page-header";
import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useAllEmployees } from "@/hooks/use-employees";
import { deleteEmployeeContract, fetchWorkspaceContracts } from "@/lib/api/contracts";
import { getStandaloneOrganizationId } from "@/lib/api/helpers";
import { ROUTES } from "@/lib/routes";

interface SelectEmployeeModalProps {
  employees: Employee[];
  isOpen: boolean;
  onClose: () => void;
  onSelect: (employeeId: string) => void;
}

function SelectEmployeeModal({ isOpen, onClose, onSelect, employees }: SelectEmployeeModalProps) {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");

  const filtered = useMemo(() => {
    return employees.filter(emp =>
      emp.full_name.toLowerCase().includes(search.toLowerCase()) ||
      emp.employee_id_number.toLowerCase().includes(search.toLowerCase())
    );
  }, [employees, search]);

  return (
    <Modal
      icon={<Plus size={24} />}
      isOpen={isOpen}
      onClose={onClose}
      subtitle={t("employees.details.contracts.select_employee_subtitle", { defaultValue: "Select an employee to sign/create a contract" })}
      title={t("employees.details.contracts.select_employee", { defaultValue: "Select Employee" })}
    >
      <div className="space-y-4 pt-4">
        <FormField
          className="!px-0"
          inputClassName="bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 dark:bg-slate-900/55 dark:border-slate-800 text-sm"
          label={t("employees.table.search_placeholder", { defaultValue: "Search Employee" })}
          onChange={(e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => { setSearch(e.target.value); }}
          placeholder={t("employees.table.search_placeholder", { defaultValue: "Search employee by name or code..." })}
          value={search}
        />

        <div className="max-h-72 overflow-y-auto space-y-1.5 pr-1">
          {filtered.length === 0 ? (
            <div className="text-center py-8 text-slate-400 text-xs">
              {t("common.no_data", { defaultValue: "No employees found" })}
            </div>
          ) : (
            filtered.map((emp) => (
              <div
                className="w-full flex items-center justify-between p-3.5 rounded-xl border border-slate-100 hover:border-primary-200 dark:border-slate-800/60 dark:hover:border-primary-900/50 hover:bg-primary-50/20 dark:hover:bg-primary-950/10 text-left cursor-pointer transition-all group focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500"
                key={emp.id}
                onClick={() => {
                  onSelect(emp.id);
                  onClose();
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    onSelect(emp.id);
                    onClose();
                  }
                }}
                role="button"
                tabIndex={0}
              >
                <div>
                  <div className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wider group-hover:text-primary-600 transition-colors">
                    {emp.full_name}
                  </div>
                  {!!emp.employee_id_number && (
                    <div className="text-[10px] font-mono text-slate-400 mt-0.5">
                      {emp.employee_id_number}
                    </div>
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </Modal>
  );
}

export default function EContractsPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const wsId = getStandaloneOrganizationId() ?? "";
  const { confirm, ConfirmationDialog } = useConfirm();

  const searchParams = useSearch({ from: "/_authenticated/_workspace/econtracts" });

  const [search, setSearch] = useState(searchParams.q ?? "");
  const [selectedType, setSelectedType] = useState<string>("all");
  const [selectedStatus, setSelectedStatus] = useState<string>("all");

  const [isSelectEmpOpen, setIsSelectEmpOpen] = useState(false);
  const [activeEmployeeId, setActiveEmployeeId] = useState<string | null>(null);
  const [isContractFormOpen, setIsContractFormOpen] = useState(false);
  const [editingContract, setEditingContract] = useState<EmployeeContract | null>(null);

  const { data: contracts = [], isLoading: isLoadingContracts } = useQuery({
    queryKey: ["workspace-contracts", wsId],
    queryFn: () => fetchWorkspaceContracts(wsId),
    enabled: !!wsId,
  });

  const { employees, isLoading: isLoadingEmployees } = useAllEmployees(wsId);

  const deleteMutation = useMutation({
    mutationFn: ({ employeeId, contractId }: { employeeId: string; contractId: string }) =>
      deleteEmployeeContract(wsId, employeeId, contractId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["workspace-contracts", wsId] });
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

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  const isPageLoading = isLoadingContracts || isLoadingEmployees;

  return (
    <PageContainer>
      <PageHeader
        actions={
          <div className="flex flex-wrap items-center gap-3">
            <Link
              className="flex items-center gap-2 text-xs font-bold text-slate-600 dark:text-slate-400 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl px-4 py-2.5 hover:bg-slate-50 dark:hover:bg-slate-850 hover:text-primary-600 transition-all shadow-sm"
              to={ROUTES.SETTINGS.ECONTRACTS}
            >
              <Settings size={14} /> CONFIGURATIONS
            </Link>
            <Button
              className="flex items-center gap-1.5 px-4 py-2.5 rounded-xl text-xs font-bold shadow-sm"
              onClick={() => {
                setEditingContract(null);
                setActiveEmployeeId(null);
                setIsSelectEmpOpen(true);
              }}
              variant="secondary"
            >
              <Plus size={16} /> SIGN CONTRACT
            </Button>
          </div>
        }
        icon={<Scale size={24} />}
        subtitle="Manage, view, and sign digital employment contracts for all employees across the organization"
        title="Centralized Contract Registry"
      />

      <EContractsFilters
        onSearchChange={setSearch}
        onStatusChange={setSelectedStatus}
        onTypeChange={setSelectedType}
        search={search}
        selectedStatus={selectedStatus}
        selectedType={selectedType}
      />

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] shadow-sm overflow-hidden relative min-h-[400px]">
        <EContractsTable
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
        employees={employees}
        isOpen={isSelectEmpOpen}
        onClose={() => { setIsSelectEmpOpen(false); }}
        onSelect={(employeeId) => {
          setActiveEmployeeId(employeeId);
          setIsContractFormOpen(true);
        }}
      />

      {isContractFormOpen && activeEmployeeId ? (
        <ContractFormModal
          editingContract={editingContract}
          employeeId={activeEmployeeId}
          isOpen={isContractFormOpen}
          onClose={() => {
            setIsContractFormOpen(false);
            setActiveEmployeeId(null);
            setEditingContract(null);
          }}
          wsId={wsId}
        />
      ) : null}

      <ConfirmationDialog />
    </PageContainer>
  );
}
