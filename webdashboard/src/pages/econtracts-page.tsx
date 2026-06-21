import {
  Scale,
  Search,
  FileText,
  Plus,
  Trash2,
  Pencil,
  Settings,
  ChevronRight,
  Filter
} from "lucide-react";
import React, { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Link, useSearch } from "@tanstack/react-router";

import PageHeader from "@/components/ui/page-header";
import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import EmptyState from "@/components/ui/empty-state";
import { useToast } from "@/components/ui/toast";
import { cn } from "@/lib/utils";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { fetchWorkspaceContracts, deleteEmployeeContract } from "@/lib/api/contracts";
import { useAllEmployees } from "@/hooks/use-employees";
import ContractFormModal from "@/components/features/employees/contract-form-modal";
import { useConfirm } from "@/hooks/use-confirm";
import type { EmployeeContract, ContractType, ContractStatus } from "@/lib/api";

interface SelectEmployeeModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (employeeId: string) => void;
  employees: any[];
}

function SelectEmployeeModal({ isOpen, onClose, onSelect, employees }: SelectEmployeeModalProps) {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");

  const filtered = useMemo(() => {
    return employees.filter(emp =>
      emp.full_name.toLowerCase().includes(search.toLowerCase()) ||
      (emp.employee_id_number && emp.employee_id_number.toLowerCase().includes(search.toLowerCase()))
    );
  }, [employees, search]);

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={t("employees.details.contracts.select_employee", { defaultValue: "Select Employee" })}
      subtitle={t("employees.details.contracts.select_employee_subtitle", { defaultValue: "Select an employee to sign/create a contract" })}
      icon={<Plus size={24} />}
    >
      <div className="space-y-4 pt-4">
        <div className="relative">
          <Search className="absolute left-3 top-3.5 text-slate-400" size={16} />
          <input
            type="text"
            placeholder={t("employees.table.search_placeholder", { defaultValue: "Search employee by name or code..." })}
            className="w-full pl-10 pr-4 h-11 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 dark:bg-slate-900/55 dark:border-slate-800 text-sm"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>

        <div className="max-h-72 overflow-y-auto space-y-1.5 pr-1">
          {filtered.length === 0 ? (
            <div className="text-center py-8 text-slate-400 text-xs">
              {t("common.no_data", { defaultValue: "No employees found" })}
            </div>
          ) : (
            filtered.map((emp) => (
              <div
                key={emp.id}
                role="button"
                tabIndex={0}
                className="w-full flex items-center justify-between p-3.5 rounded-xl border border-slate-100 hover:border-primary-200 dark:border-slate-800/60 dark:hover:border-primary-900/50 hover:bg-primary-50/20 dark:hover:bg-primary-950/10 text-left cursor-pointer transition-all group focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500"
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
              >
                <div>
                  <div className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wider group-hover:text-primary-600 transition-colors">
                    {emp.full_name}
                  </div>
                  {emp.employee_id_number && (
                    <div className="text-[10px] font-mono text-slate-400 mt-0.5">
                      {emp.employee_id_number}
                    </div>
                  )}
                </div>
                <ChevronRight size={14} className="text-slate-400 group-hover:text-primary-500 transition-colors" />
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
  const wsId = getCurrentWorkspaceId() || "";
  const { formatDate } = useDateTimeFormatter();
  const { confirm, ConfirmationDialog } = useConfirm();

  const searchParams = useSearch({ from: "/_authenticated/_workspace/econtracts" });

  const [search, setSearch] = useState(searchParams.q || "");
  const [selectedType, setSelectedType] = useState<string>("all");
  const [selectedStatus, setSelectedStatus] = useState<string>("all");

  // Sign contract flow states
  const [isSelectEmpOpen, setIsSelectEmpOpen] = useState(false);
  const [activeEmployeeId, setActiveEmployeeId] = useState<string | null>(null);
  const [isContractFormOpen, setIsContractFormOpen] = useState(false);
  const [editingContract, setEditingContract] = useState<EmployeeContract | null>(null);

  // Queries
  const { data: contracts = [], isLoading: isLoadingContracts } = useQuery({
    queryKey: ["workspace-contracts", wsId],
    queryFn: () => fetchWorkspaceContracts(wsId),
    enabled: !!wsId,
  });

  const { employees = [], isLoading: isLoadingEmployees } = useAllEmployees(wsId);

  // Mutations
  const deleteMutation = useMutation({
    mutationFn: ({ employeeId, contractId }: { employeeId: string; contractId: string }) =>
      deleteEmployeeContract(wsId, employeeId, contractId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["workspace-contracts", wsId] });
      toast.success(t("common.deleted_success", { defaultValue: "Contract deleted successfully" }));
    },
    onError: (err: any) => {
      toast.error(err.message || "Failed to delete contract");
    },
  });

  const handleDelete = async (employeeId: string, contractId: string, contractNumber?: string | null) => {
    const isConfirmed = await confirm({
      title: t("common.delete", { defaultValue: "Delete Contract" }),
      message: t("employees.details.contracts.delete_confirm", {
        defaultValue: `Are you sure you want to delete contract ${contractNumber || ""} ? This action cannot be undone.`
      }),
      confirmText: t("common.delete", { defaultValue: "Delete" }),
      cancelText: t("common.cancel", { defaultValue: "Cancel" }),
    });

    if (isConfirmed) {
      deleteMutation.mutate({ employeeId, contractId });
    }
  };

  const getContractTypeLabel = (type: ContractType) => {
    switch (type) {
      case "probation": return t("employees.details.contracts.types.probation", { defaultValue: "Probation" });
      case "training": return t("employees.details.contracts.types.training", { defaultValue: "Training" });
      case "fixed_term": return t("employees.details.contracts.types.fixed_term", { defaultValue: "Fixed Term" });
      case "indefinite": return t("employees.details.contracts.types.indefinite", { defaultValue: "Indefinite" });
      default: return type;
    }
  };

  const getStatusColor = (status: ContractStatus) => {
    switch (status) {
      case "active": return "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800";
      case "expired": return "bg-rose-50 text-rose-600 border-rose-100 dark:bg-rose-950/40 dark:text-rose-300 dark:border-rose-800";
      case "terminated": return "bg-slate-50 text-slate-500 border-slate-100 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700";
      case "draft": return "bg-amber-50 text-amber-600 border-amber-100 dark:bg-amber-950/40 dark:text-amber-300 dark:border-amber-800";
      default: return "bg-slate-50 text-slate-500";
    }
  };

  const filteredContracts = useMemo(() => {
    return contracts.filter((c) => {
      const nameMatch = c.employee_name.toLowerCase().includes(search.toLowerCase());
      const codeMatch = c.employee_code.toLowerCase().includes(search.toLowerCase());
      const numMatch = c.contract.contract_number?.toLowerCase().includes(search.toLowerCase()) ?? false;
      const matchesSearch = nameMatch || codeMatch || numMatch;

      const matchesType = selectedType === "all" || c.contract.contract_type === selectedType;
      const matchesStatus = selectedStatus === "all" || c.contract.status === selectedStatus;

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
    <div className="space-y-6 animate-in fade-in duration-700 pb-20">
      <PageHeader
        icon={<Scale size={24} />}
        subtitle="Manage, view, and sign digital employment contracts for all employees across the organization"
        title="Centralized Contract Registry"
        actions={
          <div className="flex flex-wrap items-center gap-3">
            <Link
              to="/workspace/settings/econtracts"
              className="flex items-center gap-2 text-xs font-bold text-slate-600 dark:text-slate-400 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl px-4 py-2.5 hover:bg-slate-50 dark:hover:bg-slate-850 hover:text-primary-600 transition-all shadow-sm"
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
      />

      {/* Search and Filters panel */}
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-sm flex flex-col md:flex-row items-center justify-between gap-4">
        <div className="relative w-full md:w-80">
          <Search className="absolute left-3.5 top-3 text-slate-400" size={16} />
          <input
            type="text"
            placeholder="Search employee, ID or contract..."
            className="w-full pl-10 pr-4 h-10 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 dark:bg-slate-850 dark:border-slate-800 text-xs font-medium"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <div className="w-full md:w-auto flex flex-wrap md:flex-nowrap items-center gap-3">
          {/* Contract Type select */}
          <div className="flex items-center gap-2">
            <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1">
              <Filter size={12} /> TYPE
            </span>
            <select
              className="h-10 px-3 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 dark:bg-slate-850 dark:border-slate-800 text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300"
              value={selectedType}
              onChange={(e) => setSelectedType(e.target.value)}
            >
              <option value="all">ALL TYPES</option>
              <option value="probation">PROBATION</option>
              <option value="training">TRAINING</option>
              <option value="fixed_term">FIXED TERM</option>
              <option value="indefinite">INDEFINITE</option>
            </select>
          </div>

          {/* Status select */}
          <div className="flex items-center gap-2">
            <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1">
              <Filter size={12} /> STATUS
            </span>
            <select
              className="h-10 px-3 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 dark:bg-slate-850 dark:border-slate-800 text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300"
              value={selectedStatus}
              onChange={(e) => setSelectedStatus(e.target.value)}
            >
              <option value="all">ALL STATUS</option>
              <option value="draft">DRAFT</option>
              <option value="active">ACTIVE</option>
              <option value="expired">EXPIRED</option>
              <option value="terminated">TERMINATED</option>
            </select>
          </div>
        </div>
      </div>

      {/* Contracts Registry Listing */}
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] shadow-sm overflow-hidden relative min-h-[400px]">
        {isPageLoading ? (
          <div className="p-8 space-y-4 animate-pulse">
            <div className="h-10 bg-slate-100 dark:bg-slate-800 rounded-xl w-full"></div>
            <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full"></div>
            <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full"></div>
            <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full"></div>
          </div>
        ) : filteredContracts.length === 0 ? (
          <EmptyState
            icon={<FileText size={48} />}
            title="No Contracts Found"
            description={search || selectedType !== "all" || selectedStatus !== "all" 
              ? "Try adjusting your search criteria or filters to locate specific contracts" 
              : "No employment contracts have been registered in this workspace yet"}
            action={(search || selectedType !== "all" || selectedStatus !== "all") ? (
              <Button
                variant="outline"
                onClick={() => {
                  setSearch("");
                  setSelectedType("all");
                  setSelectedStatus("all");
                }}
              >
                CLEAR FILTERS
              </Button>
            ) : undefined}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="border-b border-slate-50 dark:border-slate-800/50">
                  <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                    EMPLOYEE
                  </th>
                  <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                    CONTRACT NO.
                  </th>
                  <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                    TYPE
                  </th>
                  <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                    DURATION
                  </th>
                  <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                    STATUS
                  </th>
                  <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest text-right">
                    ACTIONS
                  </th>
                </tr>
              </thead>
              <tbody>
                {filteredContracts.map((c) => (
                  <tr
                    key={c.contract.id}
                    className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/40 dark:hover:bg-slate-850/10 transition-colors group"
                  >
                    <td className="px-8 py-5">
                      <div className="flex flex-col">
                        <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wide group-hover:text-primary-600 transition-colors">
                          {c.employee_name}
                        </span>
                        {c.employee_code && (
                          <span className="text-[10px] font-mono text-slate-400 mt-0.5">
                            {c.employee_code}
                          </span>
                        )}
                      </div>
                    </td>
                    <td className="px-8 py-5 text-xs font-mono font-semibold text-slate-600 dark:text-slate-350">
                      {c.contract.contract_number || "—"}
                    </td>
                    <td className="px-8 py-5">
                      <span className="text-xs font-bold text-slate-600 dark:text-slate-300">
                        {getContractTypeLabel(c.contract.contract_type)}
                      </span>
                    </td>
                    <td className="px-8 py-5 text-xs text-slate-500 dark:text-slate-400 font-medium">
                      {formatDate(c.contract.start_date)}
                      {" → "}
                      {c.contract.end_date ? formatDate(c.contract.end_date) : "Lâu dài / Permanent"}
                    </td>
                    <td className="px-8 py-5">
                      <span className={cn("text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 border rounded-full shadow-sm", getStatusColor(c.contract.status))}>
                        {c.contract.status}
                      </span>
                    </td>
                    <td className="px-8 py-5 text-right">
                      <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                        <Button
                          className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 dark:hover:bg-primary-900/30 rounded-xl transition-colors h-auto w-auto shadow-none"
                          onClick={() => {
                            setEditingContract(c.contract);
                            setActiveEmployeeId(c.employee_id);
                            setIsContractFormOpen(true);
                          }}
                          size="icon"
                          variant="ghost"
                        >
                          <Pencil size={14} />
                        </Button>
                        <Button
                          className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-900/30 rounded-xl transition-colors h-auto w-auto shadow-none"
                          onClick={() => {
                            void handleDelete(c.employee_id, c.contract.id, c.contract.contract_number);
                          }}
                          size="icon"
                          variant="ghost"
                        >
                          <Trash2 size={14} />
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Select Employee Flow Step Modal */}
      <SelectEmployeeModal
        isOpen={isSelectEmpOpen}
        onClose={() => setIsSelectEmpOpen(false)}
        employees={employees}
        onSelect={(employeeId) => {
          setActiveEmployeeId(employeeId);
          setIsContractFormOpen(true);
        }}
      />

      {/* Main Contract Form Modal */}
      {isContractFormOpen && activeEmployeeId && (
        <ContractFormModal
          isOpen={isContractFormOpen}
          onClose={() => {
            setIsContractFormOpen(false);
            setActiveEmployeeId(null);
            setEditingContract(null);
          }}
          editingContract={editingContract}
          wsId={wsId}
          employeeId={activeEmployeeId}
        />
      )}

      {/* Confirmation Dialog Placeholder */}
      <ConfirmationDialog />
    </div>
  );
}
