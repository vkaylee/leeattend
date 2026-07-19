import { Filter, User, X } from 'lucide-react';
import React, { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

import { useDebounce } from '../../hooks/use-debounce';
import { useEmployeeSearch } from '../../hooks/use-employee-search';
import { useEmployeeSelection } from '../../hooks/use-employee-selection';
import { Department, Employee } from '../../lib/api/types';
import { Combobox } from '../ui/combobox';

interface EmployeeSearchProps {
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
  value?: string | string[];
  workspaceId: string;
}

function highlightMatch(text: string, query: string) {
  if (!query.trim()) return text;
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escapedQuery})`, 'gi');
  return (
    <>
      {text.split(regex).map((part, idx) =>
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
    icon: <User className='h-4 w-4 text-slate-500 dark:text-slate-400' />,
  };
};

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

interface DepartmentFilterButtonProps {
  departments: Department[];
  onChange: (ids: string[]) => void;
  selectedIds: string[];
}

function DepartmentFilterButton({ departments, onChange, selectedIds }: DepartmentFilterButtonProps) {
  const { t } = useTranslation();
  const activeCount = selectedIds.length;

  const toggle = (deptId: string) => {
    if (selectedIds.includes(deptId)) {
      onChange(selectedIds.filter(id => id !== deptId));
    } else {
      onChange([...selectedIds, deptId]);
    }
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          aria-label={t('common.filter_by_department', { defaultValue: 'Filter by department' })}
          data-testid="employee-dept-filter"
          className={`relative flex h-[var(--ui-height-md)] shrink-0 items-center gap-2 rounded-2xl border px-4 text-sm font-bold uppercase tracking-tight shadow-inner transition-all ${
            activeCount > 0
              ? 'border-primary-500 dark:border-primary-500 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400'
              : 'border-slate-100 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:border-primary-200 dark:hover:border-primary-800'
          }`}
          type="button"
          variant="ghost"
        >
          <Filter className="h-4 w-4" />
          {activeCount > 0 ? (
            <span className="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary-600 dark:bg-primary-500 px-1.5 text-[10px] text-white">
              {activeCount}
            </span>
          ) : null}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="max-h-80 w-64 overflow-y-auto custom-scrollbar" sideOffset={8}>
        <DropdownMenuItem
          className={selectedIds.length === 0 ? 'text-primary-600 dark:text-primary-400' : ''}
          onSelect={(e) => { e.preventDefault(); onChange([]); }}
        >
          {t('common.all_departments', { defaultValue: 'All departments' })}
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        {departments.map((dept) => (
          <DropdownMenuCheckboxItem
            checked={selectedIds.includes(dept.id)}
            key={dept.id}
            onSelect={(e) => { e.preventDefault(); toggle(dept.id); }}
          >
            {dept.name}
          </DropdownMenuCheckboxItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function EmployeeSearch({
  workspaceId,
  value,
  onChange,
  placeholder,
  className,
  id,
  'data-testid': testId,
  multiple = false,
  hideChips = false,
  keyboardShortcut,
  clearOnEscape,
  onSelectEmployee,
  emptyQueryBehavior = 'all',
  permission,
  departments,
  selectedDepartmentIds,
  onDepartmentFilterChange,
}: EmployeeSearchProps) {
  const { t } = useTranslation();
  const [searchTerm, setSearchTerm] = useState('');
  const [internalDeptIds, setInternalDeptIds] = useState<string[]>([]);
  const debouncedSearchTerm = useDebounce(searchTerm, 300);
  const selectedIds = useMemo(() => {
    if (Array.isArray(value)) return value;
    if (value) return [value];
    return [];
  }, [value]);

  const effectiveDeptIds = selectedDepartmentIds ?? internalDeptIds;
  const handleDeptChange = (ids: string[]) => {
    if (onDepartmentFilterChange) onDepartmentFilterChange(ids);
    if (selectedDepartmentIds === undefined) setInternalDeptIds(ids);
  };

  const {
    employees,
    selectedMeta,
    setSelectedMeta,
  } = useEmployeeSearch({
    workspaceId,
    debouncedSearchTerm,
    selectedIds,
    emptyQueryBehavior,
    permission,
    departmentIds: effectiveDeptIds,
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

  const options = useMemo(() => {
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

  const hasDepartmentFilter = Boolean(departments && departments.length > 0);

  return (
    <div className={`space-y-2 ${className ?? ''}`}>
      <div className="flex items-center gap-2">
        <div className="flex-1 min-w-0">
          <Combobox
            clearOnEscape={clearOnEscape}
            data-testid={testId}
            id={id}
            keyboardShortcut={keyboardShortcut}
            onChange={handleSelect}
            onSearchChange={setSearchTerm}
            options={options}
            placeholder={placeholder ?? t('common.search')}
            value={selectedValue}
          />
        </div>
        {hasDepartmentFilter ? (
          <DepartmentFilterButton
            departments={departments ?? []}
            onChange={handleDeptChange}
            selectedIds={effectiveDeptIds}
          />
        ) : null}
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
