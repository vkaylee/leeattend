import { useQuery } from '@tanstack/react-query';
import { User, X } from 'lucide-react';
import React, { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import { useDebounce } from '../../hooks/use-debounce';
import { searchEmployees } from '../../lib/api';
import { Employee } from '../../lib/api/types';
import { Combobox } from '../ui/Combobox';

interface EmployeeSearchProps {
  className?: string;
  clearOnEscape?: boolean;
 
  'data-testid'?: string;
  id?: string;
  keyboardShortcut?: string | string[];
  multiple?: boolean;
  onChange: (value: string | string[]) => void;
  onSelectEmployee?: (employee: Employee) => void;

  placeholder?: string;
  value?: string | string[];
  workspaceId: string;
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

  keyboardShortcut,
  clearOnEscape,
  onSelectEmployee,
}: EmployeeSearchProps) {
  const { t } = useTranslation();
  const [searchTerm, setSearchTerm] = useState('');
  const debouncedSearchTerm = useDebounce(searchTerm, 300);

  // Fetch employees with debounced query
  const { data: employeesData, isLoading: _isLoading } = useQuery({
    queryKey: ['employees', workspaceId, 'search', debouncedSearchTerm],
    queryFn: () => searchEmployees(workspaceId, debouncedSearchTerm),
    enabled: !!workspaceId,
  });

 
  const employees = useMemo(() => employeesData?.data || [], [employeesData?.data]);

  // Keep track of selected employee objects to show their names in chips
  const [selectedMeta, setSelectedMeta] = useState<Employee[]>([]);

 
  const selectedIds = useMemo(() =>
    Array.isArray(value) ? value : value ? [value] : [],
    [value]
  );

  // Auto-populate selectedMeta when employees are loaded
 
  if (employees.length > 0 && selectedIds.length > 0) {
    const found = employees.filter(emp => selectedIds.includes(emp.id));
    if (found.length > 0) {
      const missing = found.filter(f => !selectedMeta.find(m => m.id === f.id));
      if (missing.length > 0) {
        setSelectedMeta(previous => [...previous, ...missing]);
      }
    }
  }

  const handleSelect = (selectedId: string) => {
    const emp = employees.find(e => e.id === selectedId);

    if (emp && !selectedMeta.find(m => m.id === emp.id)) {
      setSelectedMeta(previous => [...previous, emp]);
    }

    if (emp && onSelectEmployee) {
      onSelectEmployee(emp);
    }

    if (multiple) {
      const current = Array.isArray(value) ? value : [];
      if (!current.includes(selectedId)) {
        onChange([...current, selectedId]);
      }
      setSearchTerm(''); // Reset for next search
    } else {
      onChange(selectedId);
    }
  };

  const handleRemove = (idToRemove: string) => {
    if (multiple && Array.isArray(value)) {
      onChange(value.filter(id => id !== idToRemove));
    }
  };

  // Helper to highlight matching text in search results
  const highlightMatch = (text: string, query: string) => {
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
  };

  // Transform to Combobox options
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

  // Only show previously-selected employee at top when NOT actively searching.
  // During search, only server-filtered results should appear.
  const extraOptions = searchTerm
    ? []
    : selectedMeta
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

  const options = [...extraOptions, ...baseOptions];

  return (
    <div className={'space-y-2 ' + (className || '')}>
      <Combobox
        clearOnEscape={clearOnEscape}
        data-testid={testId}
        id={id}
        keyboardShortcut={keyboardShortcut}
        onChange={handleSelect}
        onSearchChange={setSearchTerm}
        options={options}

        placeholder={placeholder || t('common.search')}
        value={multiple ? '' : (Array.isArray(value) ? '' : (value || ''))}
 
      />

      {!!multiple && selectedIds.length > 0 && (
        <div className='flex flex-wrap gap-2'>
          {selectedIds.map(id => {
            const meta = selectedMeta.find(m => m.id === id);
            return (
              <div
                className='flex items-center gap-1.5 px-2.5 py-1 bg-slate-100 dark:bg-slate-700 text-slate-700 dark:text-slate-200 rounded-lg text-xs font-medium border border-slate-200 dark:border-slate-700'
                key={id}
              >
                <span>{meta?.full_name || id}</span>
                <button
                  className='hover:text-rose-500 transition-colors'
                  onClick={() => { handleRemove(id); }}
                  type='button'
                >
                  <X size={14} />
                </button>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
