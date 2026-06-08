import { useMemo } from "react";
 
import { Employee, Payslip } from "@/lib/api/types";

interface UsePayrollSummaryLogicProps {
  employees: Employee[];
  onToggleSelect: (id: string) => void;
  payslips: Payslip[];
  searchTerm: string;
  selectedIds: Set<string>;
}

export function usePayrollSummaryLogic({
  employees,
  payslips,
  searchTerm,
  selectedIds,
  onToggleSelect
}: UsePayrollSummaryLogicProps) {
  const payslipMap = useMemo(() => {
    const map: Record<string, Payslip> = {};
    payslips.forEach(p => {
      const emp = employees.find(e => e.active_employment_id === p.employment_id);
      if (emp) {
        map[emp.id] = p;
      }
    });
    return map;
  }, [payslips, employees]);

  const filteredEmployees = useMemo(() => {
    return employees.filter(emp => 
      emp.full_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      emp.employee_id_number.toLowerCase().includes(searchTerm.toLowerCase())
    );
  }, [employees, searchTerm]);

  const handleSelectPage = (pageIds: string[], isPageSelected: boolean) => {
    if (isPageSelected) {
      pageIds.forEach(id => {
        if (selectedIds.has(id)) onToggleSelect(id);
      });
    } else {
      pageIds.forEach(id => {
        if (!selectedIds.has(id)) onToggleSelect(id);
      });
    }
  };

  return {
    payslipMap,
    filteredEmployees,
    handleSelectPage
  };
}
