import { ChevronDown, Loader2, Users, X } from "lucide-react";
import { memo, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { SearchInput } from "@/components/ui/search-input";
import { useDebounce } from "@/hooks/use-debounce";
import { Department, Employee } from "@/lib/api/types";

import { useBulkEmployeePicker } from "./hooks/use-bulk-employee-picker";

type CheckState = "checked" | "unchecked" | "indeterminate";

// Native checkbox can't express "indeterminate" via props — it's a DOM-only
// property, so we set it through a ref whenever the tri-state changes.
function TriStateCheckbox({
  state,
  onChange,
  ariaLabel,
}: {
  state: CheckState;
  onChange: () => void;
  ariaLabel: string;
}) {
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (ref.current) {
      ref.current.indeterminate = state === "indeterminate";
    }
  }, [state]);
  return (
    <input
      aria-label={ariaLabel}
      checked={state === "checked"}
      className="rounded border-slate-300 text-primary-600 focus:ring-primary-500 cursor-pointer h-4 w-4"
      onChange={onChange}
      onClick={(e) => { e.stopPropagation(); }}
      ref={ref}
      type="checkbox"
    />
  );
}

interface DepartmentGroupProps {
  department: Department;
  picker: ReturnType<typeof useBulkEmployeePicker>;
  searchTerm: string;
}

// eslint-disable-next-line complexity -- remaining branches are CheckState tri-state logic + filter conditionals
const DepartmentGroup = memo(function DepartmentGroup({
  department,
  picker,
  searchTerm,
}: DepartmentGroupProps) {
  const { t } = useTranslation();
  const {
    draftIds,
    expandedDepts,
    deptEmployees,
    isDeptLoading,
    toggleExpand,
    toggleDepartment,
    toggleEmployee,
    selectedInDept,
  } = picker;

  const isExpanded = expandedDepts.has(department.id);
  const isLoading = isDeptLoading(department.id);
  const loaded = deptEmployees.get(department.id);
  const selectedCount = selectedInDept(department.id);
  const total = department.active_employee_count;

  const deptState: CheckState =
    selectedCount === 0
      ? "unchecked"
      : loaded && selectedCount >= loaded.length && selectedCount > 0
        ? "checked"
        : "indeterminate";

  const visibleEmployees = useMemo(() => {
    if (!loaded) return [];
    const q = searchTerm.trim().toLowerCase();
    if (!q) return loaded;
    return loaded.filter(
      (e) =>
        e.full_name.toLowerCase().includes(q) ||
        e.employee_id_number.toLowerCase().includes(q),
    );
  }, [loaded, searchTerm]);

  return (
    <div className="border border-slate-200 dark:border-slate-700 rounded-2xl overflow-hidden">
      <div className="flex items-center gap-3 px-4 py-3 bg-slate-50 dark:bg-slate-800/50">
        <TriStateCheckbox
          ariaLabel={t("shifts.monthly.select_whole_department") + ": " + department.name}
          onChange={() => {
            void toggleDepartment(department.id, deptState !== "checked");
          }}
          state={deptState}
        />
        <Button
          aria-expanded={isExpanded}
          className="flex flex-1 items-center justify-between gap-2 cursor-pointer text-left normal-case tracking-normal !h-auto !min-w-0 !px-0 !py-0 !rounded-none"
          onClick={() => { toggleExpand(department.id); }}
          type="button"
          variant="ghost"
        >
          <span className="flex items-center gap-2">
            <span className="text-sm font-bold text-slate-700 dark:text-slate-200">
              {department.name}
            </span>
            <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">
              ({total})
            </span>
            {selectedCount > 0 ? (
              <span className="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary-600 px-1.5 text-[10px] font-bold text-white">
                {selectedCount}
              </span>
            ) : null}
          </span>
          <span className="flex items-center gap-2">
            {isLoading ? <Loader2 className="animate-spin text-slate-400" size={14} /> : null}
            <ChevronDown
              className={`text-slate-400 transition-transform ${isExpanded ? "rotate-180" : ""}`}
              size={16}
            />
          </span>
        </Button>
      </div>

      {isExpanded ? (
        <div className="divide-y divide-slate-100 dark:divide-slate-800 max-h-64 overflow-y-auto custom-scrollbar">
          {isLoading && !loaded ? (
            <div className="px-4 py-3 space-y-2">
              {Array.from({ length: 4 }).map((_, i) => (
                <div className="h-6 animate-pulse rounded-lg bg-slate-100 dark:bg-slate-800" key={i} />
              ))}
            </div>
          ) : visibleEmployees.length === 0 ? (
            <div className="px-4 py-6 text-center text-xs font-medium text-slate-400">
              {t("shifts.monthly.bulk_no_results")}
            </div>
          ) : (
            visibleEmployees.map((emp) => (
              <label
                className="flex items-center gap-3 px-4 py-2.5 hover:bg-slate-50 dark:hover:bg-slate-800/40 cursor-pointer select-none"
                key={emp.id}
              >
                <input
                  checked={draftIds.has(emp.id)}
                  className="rounded border-slate-300 text-primary-600 focus:ring-primary-500 cursor-pointer h-4 w-4"
                  onChange={() => { toggleEmployee(emp); }}
                  type="checkbox"
                />
                <span className="text-sm font-medium text-slate-700 dark:text-slate-200">
                  {emp.full_name}
                </span>
                <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {emp.employee_id_number}
                </span>
              </label>
            ))
          )}
        </div>
      ) : null}
    </div>
  );
});
DepartmentGroup.displayName = "DepartmentGroup";

interface BulkEmployeePickerModalProps {
  departments: Department[] | undefined;
  initialIds: string[];
  initialMeta: Employee[];
  isOpen: boolean;
  onClose: () => void;
  onCommit: (ids: string[], meta: Employee[]) => void;
  wsId: string;
}

export function BulkEmployeePickerModal({
  departments,
  initialIds,
  initialMeta,
  isOpen,
  onClose,
  onCommit,
  wsId,
}: BulkEmployeePickerModalProps) {
  const { t } = useTranslation();
  const [searchTerm, setSearchTerm] = useState("");
  const debouncedSearch = useDebounce(searchTerm, 300);

  const picker = useBulkEmployeePicker({ wsId, isOpen, initialIds, initialMeta });
  const { draftIds, draftMeta, deselectAll } = picker;

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- reset form state on close
    if (!isOpen) setSearchTerm("");
  }, [isOpen]);

  const handleDone = () => {
    onCommit([...draftIds], draftMeta);
    onClose();
  };

  return (
    <Modal
      icon={<Users className="text-primary-600" size={20} />}
      isOpen={isOpen}
      maxWidth="3xl"
      onClose={onClose}
      subtitle={t("shifts.monthly.bulk_picker_subtitle")}
      testId="bulk-employee-picker"
      title={t("shifts.monthly.bulk_picker_title")}
    >
      <div className="flex flex-col gap-4 px-6 py-4 flex-1 min-h-0">
        <div className="flex items-center gap-3">
          <SearchInput
            containerClassName="flex-1"
            onChange={(e) => setSearchTerm(e.target.value)}
            onClear={() => setSearchTerm("")}
            placeholder={t("shifts.monthly.bulk_search_placeholder")}
            size="sm"
            value={searchTerm}
          />
          <span className="text-xs font-bold text-slate-500 uppercase tracking-widest whitespace-nowrap">
            {t("shifts.monthly.bulk_selected_count", { count: draftIds.size })}
          </span>
          <Button
            className="flex items-center gap-1.5 px-3 py-2 text-[10px] font-extrabold uppercase tracking-widest text-slate-500 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded-xl transition-all cursor-pointer disabled:opacity-40"
            disabled={draftIds.size === 0}
            onClick={deselectAll}
            type="button"
            variant="ghost"
          >
            <X size={12} /> {t("shifts.monthly.deselect_all")}
          </Button>
        </div>

        <div className="flex flex-col gap-2.5 overflow-y-auto custom-scrollbar flex-1 min-h-0 pr-1">
          {!departments ? (
            <div className="py-10 text-center text-sm font-medium text-slate-400">
              <Loader2 className="animate-spin mx-auto mb-2" size={20} />
              {t("shifts.monthly.bulk_loading_departments")}
            </div>
          ) : (
            departments.map((dept) => (
              <DepartmentGroup
                department={dept}
                key={dept.id}
                picker={picker}
                searchTerm={debouncedSearch}
              />
            ))
          )}
        </div>
      </div>

      <div className="flex items-center justify-end gap-2.5 px-6 py-4 border-t border-slate-100 dark:border-slate-800">
        <Button
          className="px-4 py-2 text-[10px] font-bold uppercase tracking-widest text-slate-500 hover:text-slate-700 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-xl transition-all cursor-pointer"
          onClick={onClose}
          type="button"
          variant="ghost"
        >
          {t("common.cancel")}
        </Button>
        <Button
          className="px-6 py-2 bg-primary-600 text-white hover:bg-primary-700 active:scale-95 rounded-xl text-[10px] font-extrabold uppercase tracking-widest transition-all cursor-pointer shadow-sm"
          data-testid="bulk-picker-done"
          onClick={handleDone}
          type="button"
          variant="ghost"
        >
          {t("shifts.monthly.done")}
        </Button>
      </div>
    </Modal>
  );
}
