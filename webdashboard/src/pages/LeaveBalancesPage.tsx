import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { fetchEmployees, fetchEmployeeLeaveBalances, getCurrentWorkspaceId, Employee, EmployeeLeaveBalance } from "@/lib/api";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Progress } from "@/components/ui/progress";
import { Search, Filter, History } from "lucide-react";
import { LeaveHistoryModal } from "@/components/leave/LeaveHistoryModal";

export default function LeaveBalancesPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [searchTerm, setSearchTerm] = useState("");
  const [selectedEmployee, setSelectedEmployee] = useState<{ id: string, name: string } | null>(null);

  // Fetch Employees
  const { data: employees, isLoading: isLoadingEmployees } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

  // Fetch Balances for ALL employees (This is inefficient, but for MVP we fetch individual balances for the list. 
  // Ideally backend should provide a bulk endpoint. For now, we can fetch on demand or iterate.
  // Better approach: Since we don't have a bulk balance endpoint yet, let's just display the employee list 
  // and fetch balance when expanding row or just fetch for all visible?
  // Let's stick to the plan: "Admin Balances View".
  // I'll create a BalanceRow component that fetches its own balance.
  
  const filteredEmployees = employees?.filter(emp => 
    emp.first_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    emp.last_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    emp.employee_id_number.includes(searchTerm)
  );

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-extrabold tracking-tight text-slate-900">Leave Balances</h1>
          <p className="text-slate-500 mt-1">Overview and management of employee leave quotas.</p>
        </div>
      </div>

      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg font-bold">Employee Balances</CardTitle>
            <div className="flex items-center gap-2">
              <div className="relative">
                <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-slate-500" />
                <Input
                  placeholder="Search employees..."
                  className="pl-9 w-[250px]"
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                />
              </div>
              <Button variant="outline" size="icon"><Filter className="h-4 w-4" /></Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="border rounded-md">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-[300px]">Employee</TableHead>
                  <TableHead>Leave Type</TableHead>
                  <TableHead>Entitlement</TableHead>
                  <TableHead>Used</TableHead>
                  <TableHead>Remaining</TableHead>
                  <TableHead className="w-[150px]">Usage</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {isLoadingEmployees ? (
                  <TableRow><TableCell colSpan={7} className="text-center py-8">Loading...</TableCell></TableRow>
                ) : filteredEmployees?.map((emp) => (
                  <EmployeeBalanceRow 
                    key={emp.id} 
                    wsId={wsId!} 
                    employee={emp} 
                    onViewHistory={() => setSelectedEmployee({ id: emp.id, name: `${emp.first_name} ${emp.last_name}` })}
                  />
                ))}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>

      {selectedEmployee && (
        <LeaveHistoryModal
          wsId={wsId!}
          employeeId={selectedEmployee.id}
          employeeName={selectedEmployee.name}
          isOpen={!!selectedEmployee}
          onClose={() => setSelectedEmployee(null)}
        />
      )}
    </div>
  );
}

function EmployeeBalanceRow({ wsId, employee, onViewHistory }: { wsId: string, employee: Employee, onViewHistory: () => void }) {
  const { data: balances } = useQuery({
    queryKey: ["leave-balances", wsId, employee.id],
    queryFn: () => fetchEmployeeLeaveBalances(wsId, employee.id),
  });

  // For simplicity, we just show the first balance type or primary one (Vacation)
  // If multiple, we might need multiple rows or a summary.
  // Let's assume 'vacation' is primary for the table view.
  const balance = balances?.find(b => b.leave_type === "vacation") || balances?.[0];

  if (!balance) {
    return (
      <TableRow>
        <TableCell>
          <div className="flex items-center gap-3">
            <Avatar className="h-9 w-9">
              <AvatarImage src={`https://api.dicebear.com/7.x/avataaars/svg?seed=${employee.id}`} />
              <AvatarFallback>{employee.first_name[0]}{employee.last_name[0]}</AvatarFallback>
            </Avatar>
            <div>
              <div className="font-medium">{employee.first_name} {employee.last_name}</div>
              <div className="text-xs text-slate-500">{employee.employee_id_number}</div>
            </div>
          </div>
        </TableCell>
        <TableCell className="text-slate-400 italic" colSpan={5}>No balance data</TableCell>
        <TableCell className="text-right">
          <Button variant="ghost" size="sm" onClick={onViewHistory}>
            <History className="h-4 w-4" />
          </Button>
        </TableCell>
      </TableRow>
    );
  }

  const total = balance.entitlement + balance.carried_over;
  const percentUsed = total > 0 ? (balance.used / total) * 100 : 0;

  return (
    <TableRow>
      <TableCell>
        <div className="flex items-center gap-3">
          <Avatar className="h-9 w-9">
            <AvatarImage src={`https://api.dicebear.com/7.x/avataaars/svg?seed=${employee.id}`} />
            <AvatarFallback>{employee.first_name[0]}{employee.last_name[0]}</AvatarFallback>
          </Avatar>
          <div>
            <div className="font-medium">{employee.first_name} {employee.last_name}</div>
            <div className="text-xs text-slate-500">{employee.employee_id_number}</div>
          </div>
        </div>
      </TableCell>
      <TableCell className="capitalize">{balance.leave_type}</TableCell>
      <TableCell>{total}</TableCell>
      <TableCell>{balance.used}</TableCell>
      <TableCell className="font-bold text-emerald-600">{balance.remaining}</TableCell>
      <TableCell>
        <div className="flex items-center gap-2">
          <Progress value={percentUsed} className="h-2 w-[60px]" />
          <span className="text-xs text-slate-500">{Math.round(percentUsed)}%</span>
        </div>
      </TableCell>
      <TableCell className="text-right">
        <Button variant="ghost" size="sm" onClick={onViewHistory} title="View History & Adjust">
          <History className="h-4 w-4 text-slate-500 hover:text-slate-900" />
        </Button>
      </TableCell>
    </TableRow>
  );
}
