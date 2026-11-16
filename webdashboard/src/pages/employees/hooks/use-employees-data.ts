import { useQueryClient } from "@tanstack/react-query";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useOrganizationInfiniteQuery, useOrganizationMutation, useOrganizationQuery } from "@/hooks/common/use-organization-query";
import { useDepartments } from "@/hooks/use-departments";
import { useMyPermissions } from "@/hooks/use-permissions";
import {
  searchEmployees,
  updateEmployee,
  Employee,
  fetchEmployeeDepartmentHistory,
  updateEmployeeEmployment,
  type UpdateEmployeeRequest,
} from "@/lib/api";
import { employeeKeys, createQueryKey  } from "@/lib/query-keys";

import type { SortKey, SortDir } from "./use-employees-state";


const PAGE_SIZE = 20;

interface UseEmployeesDataProps { customFieldFilters: Record<string, string>;
debouncedSearch: string;
filterDeptIds: string[];
filterStatus: string;
selectedEmployee: Employee | null;
setEditingEmployee?: (emp: Employee | null) => void;
setIsModalOpen?: (open: boolean) => void;
setSelectedEmployee?: (emp: Employee | null) => void;
sortDir: SortDir;
sortKey: SortKey | null;  }

export const useEmployeesData = ({
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
  const { hasPermission } = useMyPermissions();
  const canReadEmployees = hasPermission("employees:read");

  const {
    data: employeesPages,
    isLoading,
    isFetching,
    isPlaceholderData,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useOrganizationInfiniteQuery({ queryKey: employeeKeys.search({
    customFieldFilters,
    departmentIds: filterDeptIds,
    search: debouncedSearch,
    sortDir,
    sortKey,
    status: filterStatus,
  }),
  queryFn: (pageParam) => {
    const apiSortKeyMapping = { name: "name", id: "employee_id_number", department: "department" } as const;
    const apiSortKey = sortKey ? apiSortKeyMapping[sortKey] : undefined;
    const cfString = Object.entries(customFieldFilters)
      .filter(([_, val]) => !!val)
      .map(([key, val]) => `${key}:${val}`)
      .join(",");
    return searchEmployees({
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
  enabled: canReadEmployees,
  placeholderData: "keepPreviousData", });

  const employees = useMemo(() => {
    if (!employeesPages) return [];
    return employeesPages.pages.flatMap((p) => p.data);
  }, [employeesPages]);

  const totalItems = employeesPages?.pages[0]?.total ?? 0;

  const { data: departments } = useDepartments();

  const { data: deptHistory } = useOrganizationQuery({
    queryKey: ["deptHistory", selectedEmployee?.id],
    queryFn: () => {
      if (!selectedEmployee) return Promise.reject(new Error("No employee selected"));
      return fetchEmployeeDepartmentHistory(selectedEmployee.id);
    },
    enabled: !!selectedEmployee,
  });



  const updateMutation = useOrganizationMutation<
    { id: string; employee: UpdateEmployeeRequest },
    Awaited<ReturnType<typeof updateEmployee>>
  >({ keys: [employeeKeys.all()],
  mutationFn: (data) => updateEmployee(data.id, data.employee),
  onSuccess: () => {
    setIsModalOpen?.(false);
    setEditingEmployee?.(null);
    toast.success(t("common.success"));
  }, });

  const terminateMutation = useOrganizationMutation<
    { employeeId: string; employmentId: string; endDate: string },
    Awaited<ReturnType<typeof updateEmployeeEmployment>>
  >({ keys: [employeeKeys.all()],
  mutationFn: (data) => updateEmployeeEmployment(data.employeeId, data.employmentId, {
    status: "terminated",
    end_date: data.endDate,
  }),
  onSuccess: (_data, variables) => {
    void queryClient.invalidateQueries({ queryKey: createQueryKey("deptHistory") });
    void queryClient.invalidateQueries({ queryKey: createQueryKey("employee-employments", variables.employeeId) });
    if (selectedEmployee?.id === variables.employeeId) {
      setSelectedEmployee?.({
        ...selectedEmployee,
        active_employment_id: null,
      });
    }
    toast.success(t("common.success"));
  }, });

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
