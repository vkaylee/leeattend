import { FileText, Pencil, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { ContractStatus, ContractType, EmployeeContract, EmployeeContractWithEmployee } from "@/lib/api";
import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import ResponsiveTable from "@/components/ui/responsive-table";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
import { cn } from "@/lib/utils";

function EContractMobileCard({
  contract,
  formatDate,
  onDelete,
  onEdit,
  t,
}: {
  contract: EmployeeContractWithEmployee;
  formatDate: (date: string) => string;
  onDelete: (employeeId: string, contractId: string, contractNumber?: string | null) => void;
  onEdit: (contract: EmployeeContract, employeeId: string) => void;
  t: (key: string, options?: { defaultValue?: string }) => string;
}) {
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

  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900">
      {/* Header: Employee info */}
      <div className="flex items-start gap-3">
        <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-primary-100 bg-primary-50 text-primary-600 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-400">
          <FileText size={20} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">
            {contract.employee_name}
          </div>
          {!!contract.employee_code && (
            <div className="mt-1 font-mono text-[10px] font-bold uppercase tracking-wider text-slate-400">
              {contract.employee_code}
            </div>
          )}
        </div>
        <span className={cn("text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 border rounded-full shadow-sm", getStatusColor(contract.status))}>
          {contract.status}
        </span>
      </div>

      {/* Contract details */}
      <div className="mt-4 space-y-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div className="flex items-center justify-between">
          <div>
            <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">
              {t("employees.details.contracts.contract_number", { defaultValue: "CONTRACT NO." })}
            </div>
            <div className="mt-1 font-mono text-xs font-semibold text-slate-600 dark:text-slate-350">
              {contract.contract_number ?? "—"}
            </div>
          </div>
          <div className="text-right">
            <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">
              {t("employees.details.contracts.type", { defaultValue: "TYPE" })}
            </div>
            <div className="mt-1 text-xs font-bold text-slate-600 dark:text-slate-300">
              {getContractTypeLabel(contract.contract_type)}
            </div>
          </div>
        </div>

        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">
            {t("employees.details.contracts.duration", { defaultValue: "DURATION" })}
          </div>
          <div className="mt-1 text-xs font-medium text-slate-500 dark:text-slate-400">
            {formatDate(contract.start_date)}
            {" → "}
            {contract.end_date ? formatDate(contract.end_date) : t("employees.details.contracts.permanent", { defaultValue: "Lâu dài / Permanent" })}
          </div>
        </div>
      </div>

      {/* Actions */}
      <div className="mt-4 flex justify-end gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
        <Button
          aria-label={t("employees.details.contracts.edit", { defaultValue: "Edit contract" })}
          className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 dark:hover:bg-primary-900/30 rounded-xl transition-colors h-auto w-auto shadow-none"
          onClick={() => { onEdit(contract, contract.employee_id); }}
          size="icon"
          variant="ghost"
        >
          <Pencil size={14} />
        </Button>
        <Button
          aria-label={t("employees.details.contracts.delete", { defaultValue: "Delete contract" })}
          className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-900/30 rounded-xl transition-colors h-auto w-auto shadow-none"
          onClick={() => { onDelete(contract.employee_id, contract.id, contract.contract_number); }}
          size="icon"
          variant="ghost"
        >
          <Trash2 size={14} />
        </Button>
      </div>
    </div>
  );
}

export interface EContractsTableProps {
  contracts: EmployeeContractWithEmployee[];
  isLoading: boolean;
  onClearFilters: () => void;
  onDelete: (employeeId: string, contractId: string, contractNumber?: string | null) => void;
  onEdit: (contract: EmployeeContract, employeeId: string) => void;
  search: string;
  selectedStatus: string;
  selectedType: string;
}

/**
 * EContractsTable — contracts listing with desktop table + mobile cards.
 * Complexity (15) from JSX conditionals for isLoading/empty states + extensive desktop table markup.
 * Display logic extracted into EContractMobileCard. Remaining branches are all UI rendering.
 */
// eslint-disable-next-line complexity, max-lines-per-function
export function EContractsTable({
  contracts,
  isLoading,
  search,
  selectedType,
  selectedStatus,
  onClearFilters,
  onEdit,
  onDelete
}: EContractsTableProps) {
  const { t } = useTranslation();
  const { formatDate } = useDateTimeFormatter();
  const {
    onClickCapture: handleHorizontalDragClick,
    setRef: setHorizontalDragScrollRef,
  } = useHorizontalDragScroll<HTMLDivElement>();

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

  if (isLoading) {
    return (
      <div className="p-8 space-y-4 animate-pulse">
        <div className="h-10 bg-slate-100 dark:bg-slate-800 rounded-xl w-full" />
        <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full" />
        <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full" />
        <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full" />
      </div>
    );
  }

  if (contracts.length === 0) {
    return (
      <EmptyState
        action={(search || selectedType !== "all" || selectedStatus !== "all") ? (
          <Button
            onClick={onClearFilters}
            variant="outline"
          >
            CLEAR FILTERS
          </Button>
        ) : undefined}
        description={search || selectedType !== "all" || selectedStatus !== "all" 
          ? t("employees.details.contracts.search_no_results", { defaultValue: "Try adjusting your search criteria or filters to locate specific contracts" })
          : t("employees.details.contracts.empty_state", { defaultValue: "No employment contracts have been registered in this organization yet" })}
        icon={<FileText size={48} />}
        title={t("employees.details.contracts.no_contracts_found", { defaultValue: "No Contracts Found" })}
      />
    );
  }

  return (
    <ResponsiveTable<EmployeeContractWithEmployee>
      data={contracts}
      emptyState={{
        title: t("employees.details.contracts.no_contracts_found", { defaultValue: "No Contracts Found" }),
        description: search || selectedType !== "all" || selectedStatus !== "all"
          ? t("employees.details.contracts.search_no_results", { defaultValue: "Try adjusting your search criteria or filters to locate specific contracts" })
          : t("employees.details.contracts.empty_state", { defaultValue: "No employment contracts have been registered in this organization yet" }),
        icon: <FileText size={48} />,
        action: (search || selectedType !== "all" || selectedStatus !== "all") ? (
          <Button onClick={onClearFilters} variant="outline">
            CLEAR FILTERS
          </Button>
        ) : undefined,
      }}
      getCardKey={(contract) => contract.id}
      isLoading={isLoading}
      renderCard={(contract) => (
        <EContractMobileCard
          contract={contract}
          formatDate={formatDate}
          onDelete={onDelete}
          onEdit={onEdit}
          t={t}
        />
      )}
      renderDesktopTable={() => (
        <div
          className="overflow-x-auto overscroll-contain"
          onClickCapture={handleHorizontalDragClick}
          ref={setHorizontalDragScrollRef}
        >
          <table className="w-full min-w-[980px] text-left border-collapse">
            <thead>
              <tr className="border-b border-slate-50 dark:border-slate-800/50">
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("employees.details.contracts.employee", { defaultValue: "EMPLOYEE" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("employees.details.contracts.contract_number", { defaultValue: "CONTRACT NO." })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("employees.details.contracts.type", { defaultValue: "TYPE" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("employees.details.contracts.duration", { defaultValue: "DURATION" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("common.status", { defaultValue: "STATUS" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest text-right">
                  {t("employees.details.contracts.actions", { defaultValue: "ACTIONS" })}
                </th>
              </tr>
            </thead>
            <tbody>
              {contracts.map((c) => (
                <tr
                  className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/40 dark:hover:bg-slate-855/10 transition-colors group"
                  key={c.id}
                >
                  <td className="px-8 py-5">
                    <div className="flex flex-col">
                      <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wide group-hover:text-primary-600 transition-colors">
                        {c.employee_name}
                      </span>
                      {!!c.employee_code && (
                        <span className="text-[10px] font-mono text-slate-400 mt-0.5">
                          {c.employee_code}
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="px-8 py-5 text-xs font-mono font-semibold text-slate-600 dark:text-slate-350">
                    {c.contract_number ?? "—"}
                  </td>
                  <td className="px-8 py-5">
                    <span className="text-xs font-bold text-slate-600 dark:text-slate-300">
                      {getContractTypeLabel(c.contract_type)}
                    </span>
                  </td>
                  <td className="px-8 py-5 text-xs text-slate-500 dark:text-slate-400 font-medium">
                    {formatDate(c.start_date)}
                    {" → "}
                    {c.end_date ? formatDate(c.end_date) : t("employees.details.contracts.permanent", { defaultValue: "Lâu dài / Permanent" })}
                  </td>
                  <td className="px-8 py-5">
                    <span className={cn("text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 border rounded-full shadow-sm", getStatusColor(c.status))}>
                      {c.status}
                    </span>
                  </td>
                  <td className="px-8 py-5 text-right">
                    <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                      <Button
                        aria-label={t("employees.details.contracts.edit", { defaultValue: "Edit contract" })}
                        className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 dark:hover:bg-primary-900/30 rounded-xl transition-colors h-auto w-auto shadow-none"
                        onClick={() => { onEdit(c, c.employee_id); }}
                        size="icon"
                        variant="ghost"
                      >
                        <Pencil size={14} />
                      </Button>
                      <Button
                        aria-label={t("employees.details.contracts.delete", { defaultValue: "Delete contract" })}
                        className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-900/30 rounded-xl transition-colors h-auto w-auto shadow-none"
                        onClick={() => { onDelete(c.employee_id, c.id, c.contract_number); }}
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
    />
  );
}
