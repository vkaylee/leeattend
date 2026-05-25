import { useQuery } from '@tanstack/react-query';
import { User, X } from 'lucide-react';
import React, { useState, useMemo, useEffect } from 'react';
import { useTranslation } from 'react-i18next';

import { useDebounce } from '../../hooks/use-debounce';
import { searchEmployees } from '../../lib/api';
import { Employee } from '../../lib/api/types';
import { Combobox } from '../ui/combobox';

interface EmployeeSearchProps {
  className?: string;
  clearOnEscape?: boolean;
  'data-testid'?: string;
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

// Helper to highlight matching text in search results
function highlightMatch(text: string, query: string) {
  if (!query.trim()) return text;

  // Escape regex characters from query
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escapedQuery})`, 'gi');
  const parts = text.split(regex);

  return (
    <>
      {parts.map((part, index) =>
        regex.test(part) ? (
          <span className="text-primary-600 dark:text-primary-400 font-black" key={index}>{part}</span>
        ) : (
          <span key={index}>{part}</span>
        )
      )}
    </>
  );
}

interface SelectedEmployeeChipsProps {
  onRemove: (id: string) => void;
  selectedIds: string[];
  selectedMeta: Employee[];
}

function SelectedEmployeeChips({ onRemove, selectedIds, selectedMeta }: SelectedEmployeeChipsProps) {
  if (selectedIds.length === 0) return null;

  return (
    <div className='flex flex-wrap gap-2'>
      {selectedIds.map(id => {
        const meta = selectedMeta.find(m => m.id === id);
        return (
          <div
            className='flex items-center gap-1.5 px-2.5 py-1 bg-slate-100 dark:bg-slate-700 text-slate-700 dark:text-slate-200 rounded-lg text-xs font-medium border border-slate-200 dark:border-slate-700'
            key={id}
          >
            <span>{meta?.full_name ?? id}</span>
            <button
              className='hover:text-rose-500 transition-colors'
              onClick={() => { onRemove(id); }}
              type='button'
            >
              <X size={14} />
            </button>
          </div>
        );
      })}
    </div>
  );
}

/**
 * A reusable employee search component with debounced server-side filtering.
 * Supports single or multiple selection (with chips).
 */
function useEmployeeSearch(workspaceId: string, debouncedSearchTerm: string, selectedIds: string[]) {
  const { data: employeesData } = useQuery({
    queryKey: ['employees', workspaceId, 'search', debouncedSearchTerm],
    queryFn: () => searchEmployees({ wsId: workspaceId, query: debouncedSearchTerm, page: 1, limit: 100 }),
    enabled: workspaceId !== '',
  });

  const employees = useMemo(() => employeesData?.data ?? [], [employeesData?.data]);
  const [selectedMeta, setSelectedMeta] = useState<Employee[]>([]);

  useEffect(() => {
    if (employees.length === 0 || selectedIds.length === 0) return;
    const found = employees.filter(emp => selectedIds.includes(emp.id));
    if (found.length === 0) return;

    void Promise.resolve().then(() => {
      setSelectedMeta(previous => {
        const missing = found.filter(f => !previous.some(m => m.id === f.id));
        if (missing.length === 0) return previous;
        return [...previous, ...missing];
      });
    });
  }, [employees, selectedIds]);

  return { employees, selectedMeta, setSelectedMeta };
}

/**
 * A reusable employee search component with debounced server-side filtering.
 * Supports single or multiple selection (with chips).
 */
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
}: EmployeeSearchProps) {
  const { t } = useTranslation();
  const [searchTerm, setSearchTerm] = useState('');
  const debouncedSearchTerm = useDebounce(searchTerm, 300);
  const selectedIds = useMemo(() => Array.isArray(value) ? value : (value ? [value] : []), [value]);
  const { employees, selectedMeta, setSelectedMeta } = useEmployeeSearch(workspaceId, debouncedSearchTerm, selectedIds);

  const handleSelect = (selectedId: string) => {
    const emp = employees.find(e => e.id === selectedId);
    if (emp && !selectedMeta.find(m => m.id === emp.id)) {
      setSelectedMeta(previous => [...previous, emp]);
    }
    if (emp && onSelectEmployee) onSelectEmployee(emp);

    if (multiple) {
      const current = Array.isArray(value) ? value : [];
      if (!current.includes(selectedId)) onChange([...current, selectedId]);
      setSearchTerm('');
    } else {
      onChange(selectedId);
    }
  };

  const options = useMemo(() => {
    const baseOptions = employees
      .filter(emp => !multiple || !selectedIds.includes(emp.id))
      .map((emp) => ({
        value: emp.id,
        label: emp.full_name,
        displayLabel: highlightMatch(emp.full_name, debouncedSearchTerm),
        description: emp.employee_id_number,
        displayDescription: highlightMatch(emp.employee_id_number, debouncedSearchTerm),
        icon: <User className='h-4 w-4 text-slate-500 dark:text-slate-400' />,
      }));

    if (searchTerm) return baseOptions;
    const extraOptions = selectedMeta
      .filter(m => selectedIds.includes(m.id))
      .filter(m => !baseOptions.find(opt => opt.value === m.id))
      .map(m => ({
        value: m.id,
        label: m.full_name,
        displayLabel: highlightMatch(m.full_name, debouncedSearchTerm),
        description: m.employee_id_number,
        displayDescription: highlightMatch(m.employee_id_number, debouncedSearchTerm),
        icon: <User className='h-4 w-4 text-slate-500 dark:text-slate-400' />,
      }));

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
        value={multiple ? '' : (Array.isArray(value) ? '' : (value ?? ''))}
      />
      {multiple && !hideChips ? (
        <SelectedEmployeeChips
          onRemove={(idToRemove) => {
            if (Array.isArray(value)) onChange(value.filter(item => item !== idToRemove));
          }}
          selectedIds={selectedIds}
          selectedMeta={selectedMeta}
        />
      ) : null}
    </div>
  );
}
