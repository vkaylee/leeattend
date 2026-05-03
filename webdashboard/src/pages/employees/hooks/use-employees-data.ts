import { useQuery, useInfiniteQuery, useMutation, useQueryClient, keepPreviousData } from "@tanstack/react-query";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import {
  searchEmployees,
  updateEmployee,
  Employee,
  fetchDepartments,
  fetchEmployeeDepartmentHistory,
  fetchEmploymentHistory,
} from "@/lib/api";

import type { SortKey, SortDir } from "./use-employees-state";

const PAGE_SIZE = 20;

interface UseEmployeesDataProps {
  debouncedSearch: string;
  filterDeptIds: string[];
  selectedEmployee: Employee | null;
  setEditingEmployee: (emp: Employee | null) => void;
  setIsModalOpen: (open: boolean) => void;
  sortDir: SortDir;
  sortKey: SortKey | null;
  wsId: string;
}

export const useEmployeesData = ({
  wsId,
  debouncedSearch,
  sortKey,
  sortDir,
  filterDeptIds,
  selectedEmployee,
  setIsModalOpen,
  setEditingEmployee,
}: UseEmployeesDataProps) => {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const {
    data: employeesPages,
    isLoading,
    isFetching,
    isPlaceholderData,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: ["employees", wsId, debouncedSearch, sortKey, sortDir, filterDeptIds],
    queryFn: ({ pageParam }) => {
      const apiSortKeyMapping = { name: "name", id: "employee_id_number", department: "department", status: "status" } as const;
      const apiSortKey = sortKey ? apiSortKeyMapping[sortKey] : undefined;
      return searchEmployees(wsId, debouncedSearch, pageParam, PAGE_SIZE, apiSortKey, sortDir, filterDeptIds.length > 0 ? filterDeptIds : undefined);
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
      if (totalLoaded >= (lastPage.total || 0)) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId,
    placeholderData: keepPreviousData,
  });

  const employees = useMemo(() => {
    if (!employeesPages) return [];
    return employeesPages.pages.flatMap((p) => p.data);
  }, [employeesPages]);

  const totalItems = employeesPages?.pages[0]?.total ?? 0;

  const { data: departments } = useQuery({ 
    queryKey: ["departments", wsId], 
    queryFn: () => fetchDepartments(wsId), 
    enabled: !!wsId 
  });

  const { data: deptHistory } = useQuery({ 
    queryKey: ["deptHistory", selectedEmployee?.id], 
    queryFn: () => selectedEmployee ? fetchEmployeeDepartmentHistory(wsId, selectedEmployee.id) : Promise.reject(new Error("No employee selected")), 
    enabled: !!selectedEmployee 
  });

  const { data: employmentHistory } = useQuery({ 
    queryKey: ["employmentHistory", selectedEmployee?.id], 
    queryFn: () => selectedEmployee ? fetchEmploymentHistory(wsId, selectedEmployee.id) : Promise.reject(new Error("No employee selected")), 
    enabled: !!selectedEmployee 
  });

  const updateMutation = useMutation({
    mutationFn: (data: { id: string; employee: Partial<Employee> }) => updateEmployee(wsId, data.id, data.employee),
    onSuccess: () => { 
      void queryClient.invalidateQueries({ queryKey: ["employees", wsId] }); 
      setIsModalOpen(false); 
      setEditingEmployee(null); 
      toast.success(t("common.success")); 
    },
  });

  return {
    employees,
    totalItems,
    departments,
    deptHistory,
    employmentHistory,
    updateMutation,
    isLoading,
    isFetching,
    isPlaceholderData,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  };
};
