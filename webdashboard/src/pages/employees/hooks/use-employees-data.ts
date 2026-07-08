import { useQuery, useInfiniteQuery, useMutation, useQueryClient, keepPreviousData } from "@tanstack/react-query";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import type { Employment } from "@/lib/api/employees";

import { useToast } from "@/components/ui/toast";
import {
  searchEmployees,
  updateEmployee,
  Employee,
  fetchDepartments,
  fetchEmployeeDepartmentHistory,
  updateEmployeeEmployment,
} from "@/lib/api";

import type { SortKey, SortDir } from "./use-employees-state";

const PAGE_SIZE = 20;

interface UseEmployeesDataProps {
  customFieldFilters: Record<string, string>;
  debouncedSearch: string;
  filterDeptIds: string[];
  filterStatus: string;
  selectedEmployee: Employee | null;
  setEditingEmployee: (emp: Employee | null) => void;
  setIsModalOpen: (open: boolean) => void;
  setSelectedEmployee?: (emp: Employee | null) => void;
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
  filterStatus,
  selectedEmployee,
  setIsModalOpen,
  setEditingEmployee,
  setSelectedEmployee,
  customFieldFilters,
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
    queryKey: ["employees", wsId, debouncedSearch, sortKey, sortDir, filterDeptIds, customFieldFilters, filterStatus],
    queryFn: ({ pageParam }) => {
      const apiSortKeyMapping = { name: "name", id: "employee_id_number", department: "department", status: "status" } as const;
      const apiSortKey = sortKey ? apiSortKeyMapping[sortKey] : undefined;
      const cfString = Object.entries(customFieldFilters)
         .filter(([_, val]) => !!val)
         .map(([key, val]) => `${key}:${val}`)
         .join(",");
      return searchEmployees({
        wsId,
        query: debouncedSearch,
        page: pageParam,
        limit: PAGE_SIZE,
        sortBy: apiSortKey,
        sortDir,
        departmentIds: filterDeptIds.length > 0 ? filterDeptIds : undefined,
        customFieldFilters: cfString || undefined,
        status: filterStatus,
      });
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



  const updateMutation = useMutation({
    mutationFn: (data: { id: string; employee: Partial<Employee> }) => updateEmployee(wsId, data.id, data.employee),
    onSuccess: () => { 
      void queryClient.invalidateQueries({ queryKey: ["employees", wsId] }); 
      setIsModalOpen(false); 
      setEditingEmployee(null); 
      toast.success(t("common.success")); 
    },
  });

  const terminateMutation = useMutation({
    mutationFn: (data: { employeeId: string; employmentId: string; endDate: string }) =>
      updateEmployeeEmployment(wsId, data.employeeId, data.employmentId, {
        status: "terminated",
        end_date: data.endDate,
      }),
    onSuccess: (data: Employment, variables: { employeeId: string; employmentId: string; endDate: string }) => {
      void queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      void queryClient.invalidateQueries({ queryKey: ["deptHistory"] });
      void queryClient.invalidateQueries({ queryKey: ["employee-employments", wsId, variables.employeeId] });
      if (selectedEmployee?.id === variables.employeeId) {
        setSelectedEmployee?.({
          ...selectedEmployee,
          active_employment_id: null,
        });
      }
      toast.success(t("common.success"));
    },
  });

  return {
    employees,
    totalItems,
    departments,
    deptHistory,

    updateMutation,
    terminateMutation,
    isLoading,
    isFetching,
    isPlaceholderData,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  };
};
