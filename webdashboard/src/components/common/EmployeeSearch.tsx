import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { User } from "lucide-react";
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
  value?: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
  id?: string;
}

/**
 * A reusable employee search component with debounced server-side filtering.
 * Searches by both name and employee ID number.
 */
export function EmployeeSearch({
  workspaceId,
  value,
  onChange,
  placeholder,
  className,
  id,
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

  // Transform to Combobox options
  const options = employees.map((emp) => ({
    value: emp.id,
    label: `${emp.first_name} ${emp.last_name}`,
    description: emp.employee_id_number,
    icon: <User className="h-4 w-4 text-slate-400" />,
  }));

  return (
    <div className="relative w-full">
      <Combobox
        id={id}
        options={options}
        value={value}
        onChange={onChange}
        onSearchChange={setSearchTerm}
        placeholder={placeholder || t("employees.search.placeholder")}
        searchPlaceholder={t("employees.search.input_placeholder")}
        emptyText={isLoading ? t("common.loading") : t("employees.search.no_results")}
        className={className}
      />
    </div>
  );
}
