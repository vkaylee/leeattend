import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { User, X } from "lucide-react";
import { Combobox } from "../ui/Combobox";
import { useDebounce } from "../../hooks/use-debounce";
import { searchEmployees } from "../../lib/api";

interface Employee {
  id: string;
  full_name: string;
  employee_id_number: string;
}

interface EmployeeSearchProps {
  workspaceId: string;
  value?: string | string[];
  onChange: (value: any) => void;
  placeholder?: string;
  className?: string;
  id?: string;
  "data-testid"?: string;
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
  "data-testid": testId,
  multiple = false,
}: EmployeeSearchProps) {
  const { t } = useTranslation();
  const [searchTerm, setSearchTerm] = useState("");
  const debouncedSearchTerm = useDebounce(searchTerm, 400);

  // Fetch employees with debounced query
  const { data: employees = [], isLoading } = useQuery<Employee[]>({
    queryKey: ["employees", workspaceId, "search", debouncedSearchTerm],
    queryFn: () => searchEmployees(workspaceId, debouncedSearchTerm),
    enabled: !!workspaceId,
  });

  // Keep track of selected employee objects to show their names in chips
  // (Alternatively, we could fetch full details for the IDs in 'value')
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
      setSearchTerm(""); // Reset for next search
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
  // In multi-mode, we hide already selected items from the dropdown list.
  // In single-mode, we keep them visible so the user can see which one is selected.
  const baseOptions = employees
    .filter(emp => !multiple || !selectedIds.includes(emp.id)) 
    .map((emp) => ({
      value: emp.id,
      label: emp.full_name,
      description: emp.employee_id_number,
      icon: <User className="h-4 w-4 text-slate-400" />,
    }));

  // Ensure currently selected items are in the options list 
  // (even if they were filtered out by server-side search)
  // so the Combobox trigger can always find their labels.
  const extraOptions = selectedMeta
    .filter(m => selectedIds.includes(m.id))
    .filter(m => !baseOptions.find(opt => opt.value === m.id))
    .map(m => ({
      value: m.id,
      label: m.full_name,
      description: m.employee_id_number,
      icon: <User className="h-4 w-4 text-slate-400" />,
    }));

  const options = [...extraOptions, ...baseOptions];

  return (
    <div className={`space-y-2 ${className}`}>
      {multiple && selectedIds.length > 0 && (
        <div className="flex flex-wrap gap-2 px-1">
          {selectedIds.map(id => {
            const meta = selectedMeta.find(m => m.id === id);
            return (
              <span 
                key={id}
                className="inline-flex items-center gap-2 bg-white text-primary-600 px-4 py-2 rounded-2xl text-[10px] font-black uppercase tracking-tight border border-primary-100 shadow-sm animate-in fade-in zoom-in-95 duration-300"
              >
                <User size={12} className="opacity-50" />
                {meta ? meta.full_name : id.slice(0, 8)}
                <button 
                  onClick={() => handleRemove(id)}
                  className="hover:bg-rose-50 hover:text-rose-600 rounded-lg p-1 transition-all active:scale-90"
                >
                  <X className="w-3.5 h-3.5" />
                </button>
              </span>
            );
          })}
        </div>
      )}
      
      <Combobox
        id={id}
        data-testid={testId}
        options={options}
        value={multiple ? undefined : (value as string || "")}
        onChange={handleSelect}
        onSearchChange={setSearchTerm}
        placeholder={placeholder || t("employees.search.placeholder")}
        searchPlaceholder={t("employees.search.input_placeholder")}
        emptyText={isLoading ? t("common.loading") : t("employees.search.no_results")}
      />
    </div>
  );
}