import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useState } from "react";

import { searchEmployees } from "@/lib/api";
import { employeeKeys } from "@/lib/query-keys";
import { Employee, PaginatedResponse } from "@/lib/api/types";

const PAGE_LIMIT = 50;

// Load every employee of a department by walking the paginated search endpoint
// until we've accumulated `total` rows. Cached per (ws, dept) via react-query so
// re-checking a department doesn't refetch.
async function fetchAllDeptEmployees(
  wsId: string,
  departmentId: string,
): Promise<Employee[]> {
  const acc: Employee[] = [];
  let page = 1;
  for (;;) {
    const res: PaginatedResponse<Employee> = await searchEmployees({
      wsId,
      query: "",
      page,
      limit: PAGE_LIMIT,
      departmentIds: [departmentId],
    });
    acc.push(...res.data);
    if (acc.length >= res.total || res.data.length === 0) break;
    page += 1;
  }
  return acc;
}

export interface UseBulkEmployeePickerOptions {
  initialIds: string[];
  initialMeta: Employee[];
  isOpen: boolean;
  wsId: string;
}

export interface UseBulkEmployeePickerReturn {
  deptEmployees: Map<string, Employee[]>;
  deselectAll: () => void;
  draftIds: Set<string>;
  draftMeta: Employee[];
  expandedDepts: Set<string>;
  isDeptLoading: (departmentId: string) => boolean;
  selectedInDept: (departmentId: string) => number;
  toggleDepartment: (departmentId: string, checked: boolean) => Promise<void>;
  toggleEmployee: (employee: Employee) => void;
  toggleExpand: (departmentId: string) => void;
}

export function useBulkEmployeePicker({
  wsId,
  isOpen,
  initialIds,
  initialMeta,
}: UseBulkEmployeePickerOptions): UseBulkEmployeePickerReturn {
  const queryClient = useQueryClient();

  const [draftIds, setDraftIds] = useState<Set<string>>(new Set());
  const [draftMetaMap, setDraftMetaMap] = useState<Map<string, Employee>>(
    new Map(),
  );
  const [expandedDepts, setExpandedDepts] = useState<Set<string>>(new Set());
  const [deptEmployees, setDeptEmployees] = useState<Map<string, Employee[]>>(
    new Map(),
  );
  const [loadingDepts, setLoadingDepts] = useState<Set<string>>(new Set());

  // Reset draft from the committed selection each time the modal opens, so
  // cancelling always reverts to what was live before opening.
  useEffect(() => {
    if (!isOpen) return;
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setDraftIds(new Set(initialIds));
    setDraftMetaMap(new Map(initialMeta.map((e) => [e.id, e])));
    setExpandedDepts(new Set());
  }, [isOpen, initialIds, initialMeta]);

  const loadDept = useCallback(
    async (departmentId: string): Promise<Employee[]> => {
      const cached = deptEmployees.get(departmentId);
      if (cached) return cached;
      setLoadingDepts((prev) => new Set(prev).add(departmentId));
      try {
        const emps = await queryClient.fetchQuery({
          queryKey: employeeKeys.lookup(wsId, {
            departmentIds: [departmentId],
            emptyQueryBehavior: "all",
            permission: "bulk-employee-picker",
          }),
          queryFn: () => fetchAllDeptEmployees(wsId, departmentId),
        });
        setDeptEmployees((prev) => new Map(prev).set(departmentId, emps));
        return emps;
      } finally {
        setLoadingDepts((prev) => {
          const next = new Set(prev);
          next.delete(departmentId);
          return next;
        });
      }
    },
    [deptEmployees, queryClient, wsId],
  );

  const toggleExpand = useCallback(
    (departmentId: string) => {
      setExpandedDepts((prev) => {
        const next = new Set(prev);
        if (next.has(departmentId)) {
          next.delete(departmentId);
        } else {
          next.add(departmentId);
          void loadDept(departmentId);
        }
        return next;
      });
    },
    [loadDept],
  );

  const addEmployees = useCallback((emps: Employee[]) => {
    setDraftIds((prev) => {
      const next = new Set(prev);
      emps.forEach((e) => next.add(e.id));
      return next;
    });
    setDraftMetaMap((prev) => {
      const next = new Map(prev);
      emps.forEach((e) => next.set(e.id, e));
      return next;
    });
  }, []);

  const removeEmployees = useCallback((ids: string[]) => {
    setDraftIds((prev) => {
      const next = new Set(prev);
      ids.forEach((id) => next.delete(id));
      return next;
    });
    setDraftMetaMap((prev) => {
      const next = new Map(prev);
      ids.forEach((id) => next.delete(id));
      return next;
    });
  }, []);

  const toggleDepartment = useCallback(
    async (departmentId: string, checked: boolean) => {
      const emps = await loadDept(departmentId);
      if (checked) {
        addEmployees(emps);
      } else {
        removeEmployees(emps.map((e) => e.id));
      }
    },
    [loadDept, addEmployees, removeEmployees],
  );

  const toggleEmployee = useCallback(
    (employee: Employee) => {
      if (draftIds.has(employee.id)) {
        removeEmployees([employee.id]);
      } else {
        addEmployees([employee]);
      }
    },
    [draftIds, addEmployees, removeEmployees],
  );

  const deselectAll = useCallback(() => {
    setDraftIds(new Set());
    setDraftMetaMap(new Map());
  }, []);

  const isDeptLoading = useCallback(
    (departmentId: string) => loadingDepts.has(departmentId),
    [loadingDepts],
  );

  const selectedInDept = useCallback(
    (departmentId: string) => {
      const emps = deptEmployees.get(departmentId);
      if (!emps) return 0;
      return emps.reduce((n, e) => (draftIds.has(e.id) ? n + 1 : n), 0);
    },
    [deptEmployees, draftIds],
  );

  const draftMeta = useMemo(() => [...draftMetaMap.values()], [draftMetaMap]);

  return {
    draftIds,
    draftMeta,
    expandedDepts,
    deptEmployees,
    isDeptLoading,
    toggleExpand,
    toggleDepartment,
    toggleEmployee,
    deselectAll,
    selectedInDept,
  };
}
