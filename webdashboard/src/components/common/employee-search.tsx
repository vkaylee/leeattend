import { User, X } from 'lucide-react';
import React, { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import type { ControlSize } from "@/components/ui/control-styles";

import Button from "@/components/ui/button";
import { useMyPermissions } from '@/hooks/use-permissions';

import { useDebounce } from '../../hooks/use-debounce';
import { useEmployeeSearch } from '../../hooks/use-employee-search';
import { useEmployeeSelection } from '../../hooks/use-employee-selection';
import { Department, Employee } from '../../lib/api/types';
import { Combobox } from '../ui/combobox';
import { DepartmentFilter } from "./department-filter";

export interface EmployeeSearchProps {
  className?: string;
  clearOnEscape?: boolean;
  'data-testid'?: string;
  departments?: Department[];
  emptyQueryBehavior?: 'prompt-to-type' | 'all';
  hideChips?: boolean;
  id?: string;
  keyboardShortcut?: string | string[];
  multiple?: boolean;
  onChange: (value: string | string[]) => void;
  onDepartmentFilterChange?: (ids: string[]) => void;
  onSelectEmployee?: (employee: Employee) => void;
  permission?: string;
  placeholder?: string;
  selectedDepartmentIds?: string[];
  size?: ControlSize;
  value?: string | string[];
}

function highlightMatch(text: string, query: string) {
  if (!query.trim()) return text;
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escapedQuery})`, 'i');
  return (
    <>
      {text.split(new RegExp(`(${escapedQuery})`, 'gi')).map((part, idx) =>
        regex.test(part) ? (
          <span className="text-primary-600 dark:text-primary-400 font-black" key={idx}>{part}</span>
        ) : (
          <span key={idx}>{part}</span>
        )
      )}
    </>
  );
}

const mapToOption = (emp: Employee, debouncedSearchTerm: string) => {
  const code = emp.employee_id_number;
  const dept = emp.department_name;
  return {
    value: emp.id,
    label: emp.full_name,
    displayLabel: highlightMatch(emp.full_name, debouncedSearchTerm),
    description: dept ? `${code} · ${dept}` : code,
    displayDescription: (
      <>
        {highlightMatch(code, debouncedSearchTerm)}
        {dept ? <span className='opacity-70'>{` · ${dept}`}</span> : null}
      </>
    ),
    icon: <User aria-hidden="true" className='h-4 w-4 text-slate-500 dark:text-slate-400' />,
  };
};

function useEmployeeSearchOptions({
  employees,
  selectedMeta,
  selectedIds,
  multiple,
  debouncedSearchTerm,
  searchTerm,
}: {
  debouncedSearchTerm: string;
  employees: Employee[];
  multiple: boolean;
  searchTerm: string;
  selectedIds: string[];
  selectedMeta: Employee[];
}) {
  return useMemo(() => {
    const baseOptions = employees
      .filter(emp => !multiple || !selectedIds.includes(emp.id))
      .map((emp) => mapToOption(emp, debouncedSearchTerm));

    if (searchTerm) return baseOptions;
    const extraOptions = selectedMeta
      .filter(m => selectedIds.includes(m.id))
      .filter(m => !baseOptions.some(opt => opt.value === m.id))
      .map(m => mapToOption(m, debouncedSearchTerm));

    return [...extraOptions, ...baseOptions];
  }, [employees, multiple, selectedIds, debouncedSearchTerm, searchTerm, selectedMeta]);
}

function useNormalizedIds(value: string | string[] | undefined): string[] {
  return useMemo(() => {
    if (Array.isArray(value)) return value;
    if (value) return [value];
    return [];
  }, [value]);
}

interface SelectedEmployeeChipsProps {
  onRemove: (id: string) => void;
  selectedIds: string[];
  selectedMeta: Employee[];
}

function SelectedEmployeeChips({ onRemove, selectedIds, selectedMeta }: SelectedEmployeeChipsProps) {
  const { t } = useTranslation();
  if (selectedIds.length === 0) return null;
  return (
    <div className='flex flex-wrap gap-2'>
      {selectedIds.map(id => {
        const meta = selectedMeta.find(m => m.id === id);
        return (
          <div className='flex items-center gap-1.5 px-2.5 py-1 bg-slate-100 dark:bg-slate-700 text-slate-700 dark:text-slate-200 rounded-lg text-xs font-medium border border-slate-200 dark:border-slate-700' key={id}>
            <span>{meta?.full_name ?? id}</span>
            <Button
              aria-label={t("common.remove", { defaultValue: "Remove" }) + " " + (meta?.full_name ?? id)}
              className='hover:text-rose-500 transition-colors'
              onClick={() => { onRemove(id); }}
              type='button'
              variant="destructive"
            >
              <X size={14} />
            </Button>
          </div>
        );
      })}
    </div>
  );
}

type NormalizedEmployeeSearchProps = Omit<
  EmployeeSearchProps,
  "departments" | "emptyQueryBehavior" | "hideChips" | "multiple" | "size"
> & {
  departments: Department[];
  emptyQueryBehavior: "prompt-to-type" | "all";
  hideChips: boolean;
  multiple: boolean;
  size: ControlSize;
};

function normalizeEmployeeSearchProps({
  departments,
  emptyQueryBehavior,
  hideChips,
  multiple,
  size,
  ...props
}: EmployeeSearchProps): NormalizedEmployeeSearchProps {
  return {
    ...props,
    departments: departments ?? [],
    emptyQueryBehavior: emptyQueryBehavior ?? "all",
    hideChips: hideChips ?? false,
    multiple: multiple ?? false,
    size: size ?? "md",
  };
}

export function EmployeeSearch(props: EmployeeSearchProps) {
  return <EmployeeSearchContent {...normalizeEmployeeSearchProps(props)} />;
}

function EmployeeSearchContent({
  value,
  onChange,
  placeholder,
  className,
  id,
  'data-testid': testId,
  multiple,
  hideChips,
  keyboardShortcut,
  clearOnEscape,
  onSelectEmployee,
  emptyQueryBehavior,
  permission,
  departments,
  selectedDepartmentIds,
  onDepartmentFilterChange,
  size,
}: NormalizedEmployeeSearchProps) {
  const { t } = useTranslation();
  const { hasPermission } = useMyPermissions();
  const [searchTerm, setSearchTerm] = useState('');
  const debouncedSearchTerm = useDebounce(searchTerm, 300);
  const selectedIds = useNormalizedIds(value);

  const [internalDeptIds, setInternalDeptIds] = useState<string[]>([]);
  const effectiveDeptIds = selectedDepartmentIds ?? internalDeptIds;
  const handleDeptChange = (ids: string[]) => {
    if (onDepartmentFilterChange) onDepartmentFilterChange(ids);
    if (selectedDepartmentIds === undefined) setInternalDeptIds(ids);
  };

  const {
    employees,
    selectedMeta,
    setSelectedMeta,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useEmployeeSearch({
    debouncedSearchTerm,
    selectedIds,
    emptyQueryBehavior,
    permission,
    departmentIds: effectiveDeptIds,
    enabled: hasPermission("employees:read"),
  });

  const { selectedValue, handleSelect } = useEmployeeSelection({
    value,
    onChange,
    employees,
    selectedMeta,
    setSelectedMeta,
    onSelectEmployee,
    multiple,
  });

  const options = useEmployeeSearchOptions({
    employees,
    selectedMeta,
    selectedIds,
    multiple,
    debouncedSearchTerm,
    searchTerm,
  });

  const departmentFilter = departments.length > 0 ? (
    <DepartmentFilter
      departments={departments}
      effectiveDeptIds={effectiveDeptIds}
      handleDeptChange={handleDeptChange}
      size={size}
      t={t}
    />
  ) : null;

  return (
    <div className={`space-y-2 ${className ?? ''}`}>
      <div className="flex items-center gap-2">
        <div className="flex-1 min-w-0">
          <Combobox
            className="border-slate-200 bg-white text-slate-700 shadow-sm hover:border-primary-300 hover:bg-white dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
            clearOnEscape={clearOnEscape}
            contentClassName="rounded-[1.5rem] border-slate-200 p-2 shadow-xl shadow-slate-900/10 dark:border-slate-700"
            data-testid={testId}
            hasMore={hasNextPage}
            id={id}
            isLoadingMore={isFetchingNextPage}
            keyboardShortcut={keyboardShortcut}
            onChange={handleSelect}
            onLoadMore={() => { void fetchNextPage(); }}
            onSearchChange={setSearchTerm}
            optionClassName="rounded-xl px-3 py-3 text-sm normal-case tracking-normal"
            options={options}
            placeholder={placeholder ?? t('common.search')}
            searchInputClassName="h-11 rounded-xl border border-slate-200 bg-slate-50 px-3 font-medium dark:border-slate-700 dark:bg-slate-800"
            size={size}
            value={selectedValue}
          />
        </div>
        {departmentFilter}
      </div>

      {multiple && !hideChips ? (
        <SelectedEmployeeChips
          onRemove={(idToRemove) => {
            const current = Array.isArray(value) ? value : [];
            onChange(current.filter((item) => item !== idToRemove));
          }}
          selectedIds={selectedIds}
          selectedMeta={selectedMeta}
        />
      ) : null}
    </div>
  );
}
