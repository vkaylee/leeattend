import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { User, X } from 'lucide-react';
import { Combobox } from '../ui/Combobox';
import { useDebounce } from '../../hooks/use-debounce';
import { searchEmployees } from '../../lib/api';
import { Employee } from '../../lib/api/types';

interface EmployeeSearchProps {
  workspaceId: string;
  value?: string | string[];
  onChange: (value: any) => void;
  placeholder?: string;
  className?: string;
  id?: string;
  'data-testid'?: string;
  multiple?: boolean;
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
}: EmployeeSearchProps) {
  const { t } = useTranslation();
  const [searchTerm, setSearchTerm] = useState('');
  const debouncedSearchTerm = useDebounce(searchTerm, 400);

  // Fetch employees with debounced query
  const { data: employeesData, isLoading } = useQuery({
    queryKey: ['employees', workspaceId, 'search', debouncedSearchTerm],
    queryFn: () => searchEmployees(workspaceId, debouncedSearchTerm),
    enabled: !!workspaceId,
  });

  const employees = employeesData?.data || [];

  // Keep track of selected employee objects to show their names in chips
  const [selectedMeta, setSelectedMeta] = useState<Employee[]>([]);

  const selectedIds = React.useMemo(() => 
    Array.isArray(value) ? value : value ? [value] : [],
    [value]
  );

  // Auto-populate selectedMeta when employees are loaded
  React.useEffect(() => {
    if (employees.length > 0 && selectedIds.length > 0) {
      const found = employees.filter(emp => selectedIds.includes(emp.id));
      if (found.length > 0) {
        setSelectedMeta(prev => {
          const missing = found.filter(f => !prev.find(m => m.id === f.id));
          if (missing.length === 0) return prev;
          return [...prev, ...missing];
        });
      }
    }
  }, [employees, selectedIds]);

  const handleSelect = (selectedId: string) => {
    const emp = employees.find(e => e.id === selectedId);
    
    if (emp && !selectedMeta.find(m => m.id === emp.id)) {
      setSelectedMeta(prev => [...prev, emp]);
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

  // Transform to Combobox options
  const baseOptions = employees
    .filter(emp => !multiple || !selectedIds.includes(emp.id)) 
    .map((emp) => ({
      value: emp.id,
      label: emp.full_name,
      description: emp.employee_id_number,
      icon: <User className='h-4 w-4 text-slate-400' />,
    }));

  const extraOptions = selectedMeta
    .filter(m => selectedIds.includes(m.id))
    .filter(m => !baseOptions.find(opt => opt.value === m.id))
    .map(m => ({
      value: m.id,
      label: m.full_name,
      description: m.employee_id_number,
      icon: <User className='h-4 w-4 text-slate-400' />,
    }));

  const options = [...extraOptions, ...baseOptions];

  return (
    <div className={'space-y-2 ' + (className || '')}>
      <Combobox
        id={id}
        data-testid={testId}
        options={options}
        value={multiple ? '' : (Array.isArray(value) ? '' : (value || ''))}
        onChange={handleSelect}
        placeholder={placeholder || t('common.search')}
        onSearchChange={setSearchTerm}
      />

      {multiple && selectedIds.length > 0 && (
        <div className='flex flex-wrap gap-2'>
          {selectedIds.map(id => {
            const meta = selectedMeta.find(m => m.id === id);
            return (
              <div
                key={id}
                className='flex items-center gap-1.5 px-2.5 py-1 bg-slate-100 text-slate-700 rounded-lg text-xs font-medium border border-slate-200'
              >
                <span>{meta?.full_name || id}</span>
                <button
                  type='button'
                  onClick={() => handleRemove(id)}
                  className='hover:text-rose-500 transition-colors'
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
