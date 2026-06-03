import { useMemo } from 'react';

import { Employee } from '../lib/api/types';

interface UseEmployeeSelectionProps {
  employees: Employee[];
  multiple?: boolean;
  onChange: (value: string | string[]) => void;
  onSelectEmployee?: (employee: Employee) => void;
  selectedMeta: Employee[];
  setSelectedMeta: React.Dispatch<React.SetStateAction<Employee[]>>;
  value: string | string[] | undefined;
}

export function useEmployeeSelection({
  value,
  onChange,
  employees,
  selectedMeta,
  setSelectedMeta,
  onSelectEmployee,
  multiple = false,
}: UseEmployeeSelectionProps) {
  const selectedValue = useMemo(() => {
    if (multiple) return '';
    if (Array.isArray(value)) return '';
    return value ?? '';
  }, [multiple, value]);

  const handleSelect = (selectedId: string) => {
    const emp = employees.find(e => e.id === selectedId);
    if (emp) {
      if (!selectedMeta.some(m => m.id === emp.id)) {
        setSelectedMeta(prev => [...prev, emp]);
      }
      if (onSelectEmployee) onSelectEmployee(emp);
    }

    if (multiple) {
      const current = Array.isArray(value) ? value : [];
      if (!current.includes(selectedId)) {
        onChange([...current, selectedId]);
      }
    } else {
      onChange(selectedId);
    }
  };

  return { selectedValue, handleSelect };
}
