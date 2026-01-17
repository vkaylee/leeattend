import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { User, X } from "lucide-react";
import { Combobox } from "../ui/Combobox";
import { useDebounce } from "../../hooks/use-debounce";
import { apiFetch } from "../../lib/api";

const API_BASE_URL = import.meta.env.VITE_API_URL || "/api";

interface Employee {
  id: string;
  first_name: string;
  last_name: string;
  employee_id_number: string;
}

interface EmployeeSearchProps {
  workspaceId: string;
  value?: string | string[];
  onChange: (value: any) => void;
  placeholder?: string;
  className?: string;
  id?: string;
  testId?: string;
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
  testId,
  multiple = false,
}: EmployeeSearchProps) {
  const { t } = useTranslation();
  const [searchTerm, setSearchTerm] = useState("");
  const debouncedSearchTerm = useDebounce(searchTerm, 400);

  // Fetch employees with debounced query
  const { data: employees = [], isLoading } = useQuery<Employee[]>({
    queryKey: ["employees", workspaceId, "search", debouncedSearchTerm],
    queryFn: async () => {
      const queryParams = new URLSearchParams();
      if (debouncedSearchTerm) {
        queryParams.append("q", debouncedSearchTerm);
      }
      
      const response = await apiFetch(
        `${API_BASE_URL}/workspaces/${workspaceId}/employees?${queryParams.toString()}`
      );
      
      if (!response.ok) {
        throw new Error("Failed to fetch employees");
      }
      return response.json();
    },
    enabled: !!workspaceId,
  });

  // Keep track of selected employee objects to show their names in chips
  // (Alternatively, we could fetch full details for the IDs in 'value')
  const [selectedMeta, setSelectedMeta] = useState<Employee[]>([]);

  const selectedIds = Array.isArray(value) ? value : value ? [value] : [];

  const handleSelect = (selectedId: string) => {
    const emp = employees.find(e => e.id === selectedId);
    
    if (multiple) {
      const current = Array.isArray(value) ? value : [];
      if (!current.includes(selectedId)) {
        onChange([...current, selectedId]);
        if (emp && !selectedMeta.find(m => m.id === emp.id)) {
          setSelectedMeta(prev => [...prev, emp]);
        }
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
  const options = employees
    .filter(emp => !selectedIds.includes(emp.id)) // Hide already selected in multi-mode
    .map((emp) => ({
      value: emp.id,
      label: `${emp.first_name} ${emp.last_name}`,
      description: emp.employee_id_number,
      icon: <User className="h-4 w-4 text-slate-400" />,
    }));

  return (
    <div className={`space-y-2 ${className}`}>
      {multiple && selectedIds.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {selectedIds.map(id => {
            const meta = selectedMeta.find(m => m.id === id);
            return (
              <span 
                key={id}
                className="inline-flex items-center gap-1.5 bg-primary-50 text-primary-700 px-3 py-1.5 rounded-full text-xs font-bold border border-primary-100 animate-in fade-in zoom-in duration-200"
              >
                {meta ? `${meta.first_name} ${meta.last_name}` : id.slice(0, 8)}
                <button 
                  onClick={() => handleRemove(id)}
                  className="hover:bg-primary-200 rounded-full p-0.5 transition-colors"
                >
                  <X className="w-3 h-3" />
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
