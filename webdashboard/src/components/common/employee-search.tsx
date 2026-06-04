import { User, X } from 'lucide-react';
import React, { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import Button from "@/components/ui/button";

import { useDebounce } from '../../hooks/use-debounce';
import { useEmployeeSearch } from '../../hooks/use-employee-search';
import { useEmployeeSelection } from '../../hooks/use-employee-selection';
import { Employee } from '../../lib/api/types';
import { Combobox } from '../ui/combobox';

interface EmployeeSearchProps {
  className?: string;
  clearOnEscape?: boolean;
  'data-testid'?: string;
  emptyQueryBehavior?: 'prompt-to-type' | 'all';
  hideChips?: boolean;
  id?: string;
  keyboardShortcut?: string | string[];
  multiple?: boolean;
  onChange: (value: string | string[]) => void;
  onSelectEmployee?: (employee: Employee) => void;
  placeholder?: string;
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

const mapToOption = (emp: Employee, debouncedSearchTerm: string) => ({
  value: emp.id,
  label: emp.full_name,
  displayLabel: highlightMatch(emp.full_name, debouncedSearchTerm),
  description: emp.employee_id_number,
  displayDescription: highlightMatch(emp.employee_id_number, debouncedSearchTerm),
  icon: <User className='h-4 w-4 text-slate-500 dark:text-slate-400' />,
});

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
}: EmployeeSearchProps) {
  const { t } = useTranslation();
  const [searchTerm, setSearchTerm] = useState('');
  const debouncedSearchTerm = useDebounce(searchTerm, 300);
  const selectedIds = useMemo(() => {
    if (Array.isArray(value)) return value;
    if (value) return [value];
    return [];
  }, [value]);

  const {
    employees,
    selectedMeta,
    setSelectedMeta,
  } = useEmployeeSearch(workspaceId, debouncedSearchTerm, selectedIds, emptyQueryBehavior);

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

  return (
    <div className={`space-y-2 ${className ?? ''}`}>
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
