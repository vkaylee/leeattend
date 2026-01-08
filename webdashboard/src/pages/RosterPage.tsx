import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Calendar as CalendarIcon, 
  ChevronLeft, 
  ChevronRight, 
  Clock, 
  User, 
  Save,
  Plus
} from "lucide-react";
import { 
  fetchEmployees, 
  fetchShifts, 
  fetchEmployeeRosters, 
  upsertRoster, 
  getCurrentWorkspaceId,
  Shift,
  Employee
} from "../lib/api";

export default function RosterPage() {
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const [currentDate, setCurrentDate] = useState(new Date());
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);

  const { data: employees } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId),
    enabled: !!wsId,
  });

  const { data: shifts } = useQuery({
    queryKey: ["shifts", wsId],
    queryFn: () => fetchShifts(wsId),
    enabled: !!wsId,
  });

  // Calculate week dates
  const startOfWeek = new Date(currentDate);
  startOfWeek.setDate(currentDate.getDate() - currentDate.getDay() + 1); // Monday

  const weekDays = Array.from({ length: 7 }, (_, i) => {
    const d = new Date(startOfWeek);
    d.setDate(startOfWeek.getDate() + i);
    return d;
  });

  const { data: rosters } = useQuery({
    queryKey: ["rosters", selectedEmployee?.id, startOfWeek.toISOString().split('T')[0]],
    queryFn: () => fetchEmployeeRosters(
      wsId, 
      selectedEmployee!.id, 
      weekDays[0].toISOString().split('T')[0],
      weekDays[6].toISOString().split('T')[0]
    ),
    enabled: !!selectedEmployee && !!wsId,
  });

  const rosterMutation = useMutation({
    mutationFn: (data: any) => upsertRoster(wsId, selectedEmployee!.id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["rosters", selectedEmployee?.id] });
    },
  });

  const handleShiftAssign = (date: Date, shiftId: string | null) => {
    if (!selectedEmployee) return;
    rosterMutation.mutate({
      employee_id: selectedEmployee.id,
      date: date.toISOString().split('T')[0],
      shift_id: shiftId,
    });
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-end">
        <div>
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">Lịch trình làm việc</h1>
          <p className="text-slate-500 mt-1">Sắp xếp ca làm việc hàng tuần cho nhân sự</p>
        </div>
        <div className="flex items-center gap-4 bg-white p-2 rounded-2xl border border-slate-200 shadow-sm">
          <button 
            onClick={() => {
              const d = new Date(currentDate);
              d.setDate(d.getDate() - 7);
              setCurrentDate(d);
            }}
            className="p-2 hover:bg-slate-50 rounded-xl transition-colors"
          >
            <ChevronLeft className="w-5 h-5" />
          </button>
          <div className="font-bold px-4">
            Tuần: {weekDays[0].toLocaleDateString('vi-VN')} - {weekDays[6].toLocaleDateString('vi-VN')}
          </div>
          <button 
            onClick={() => {
              const d = new Date(currentDate);
              d.setDate(d.getDate() + 7);
              setCurrentDate(d);
            }}
            className="p-2 hover:bg-slate-50 rounded-xl transition-colors"
          >
            <ChevronRight className="w-5 h-5" />
          </button>
        </div>
      </div>

      <div className="grid grid-cols-4 gap-6">
        {/* Employee List */}
        <div className="col-span-1 bg-white border border-slate-200 rounded-3xl overflow-hidden flex flex-col shadow-sm">
          <div className="p-4 border-b border-slate-100 bg-slate-50/50 font-bold text-sm text-slate-600 uppercase tracking-wider">
            Nhân viên
          </div>
          <div className="flex-1 overflow-y-auto max-h-[600px] divide-y divide-slate-50">
            {employees?.map(emp => (
              <button
                key={emp.id}
                onClick={() => setSelectedEmployee(emp)}
                className={`w-full flex items-center gap-3 p-4 hover:bg-blue-50/50 transition-all text-left group ${selectedEmployee?.id === emp.id ? 'bg-blue-50' : ''}`}
              >
                <div className={`w-10 h-10 rounded-full flex items-center justify-center font-bold ${selectedEmployee?.id === emp.id ? 'bg-blue-600 text-white' : 'bg-slate-100 text-slate-500'}`}>
                  {emp.last_name[0]}
                </div>
                <div>
                  <div className={`font-bold text-sm ${selectedEmployee?.id === emp.id ? 'text-blue-600' : 'text-slate-900'}`}>{emp.first_name} {emp.last_name}</div>
                  <div className="text-[10px] text-slate-500 font-mono">{emp.employee_id_number}</div>
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* Weekly Grid */}
        <div className="col-span-3 space-y-4">
          {!selectedEmployee ? (
            <div className="bg-slate-50 border-2 border-dashed border-slate-200 rounded-[40px] h-[500px] flex flex-col items-center justify-center text-center p-12">
              <User className="w-16 h-16 text-slate-300 mb-4" />
              <h3 className="text-xl font-bold text-slate-900">Chọn nhân viên</h3>
              <p className="text-slate-500 mt-2 max-w-xs">Chọn một nhân viên từ danh sách bên trái để bắt đầu sắp xếp lịch làm việc.</p>
            </div>
          ) : (
            <div className="grid grid-cols-7 gap-4">
              {weekDays.map((day, idx) => {
                const dateStr = day.toISOString().split('T')[0];
                const roster = rosters?.find(r => r.date === dateStr);
                const assignedShift = shifts?.find(s => s.id === roster?.shift_id);
                
                return (
                  <div key={idx} className="bg-white border border-slate-200 rounded-3xl p-4 flex flex-col min-h-[300px] shadow-sm hover:shadow-md transition-shadow">
                    <div className="text-center mb-4 pb-4 border-b border-slate-50">
                      <div className="text-[10px] font-bold text-slate-400 uppercase tracking-tighter">
                        {day.toLocaleDateString('vi-VN', { weekday: 'short' })}
                      </div>
                      <div className={`text-xl font-black mt-1 ${day.toDateString() === new Date().toDateString() ? 'text-blue-600' : 'text-slate-900'}`}>
                        {day.getDate()}
                      </div>
                    </div>

                    <div className="flex-1 space-y-2">
                      <div className="text-[10px] font-bold text-slate-400 uppercase mb-2">Ca làm việc</div>
                      
                      {assignedShift ? (
                        <div className="bg-blue-50 border border-blue-100 p-3 rounded-2xl relative group">
                          <div className="font-bold text-blue-700 text-xs mb-1">{assignedShift.name}</div>
                          <div className="flex items-center gap-1 text-[10px] text-blue-500">
                            <Clock className="w-3 h-3" />
                            {assignedShift.start_time.substring(0, 5)} - {assignedShift.end_time.substring(0, 5)}
                          </div>
                          <button 
                            onClick={() => handleShiftAssign(day, null)}
                            className="absolute -top-2 -right-2 bg-white border border-slate-200 rounded-full p-1 shadow-sm opacity-0 group-hover:opacity-100 transition-opacity"
                          >
                            <X size={12} className="text-slate-400" />
                          </button>
                        </div>
                      ) : (
                        <div className="bg-slate-50 border border-slate-100 p-3 rounded-2xl text-[10px] text-slate-400 font-bold text-center border-dashed">
                          NGHỈ
                        </div>
                      )}
                    </div>

                    <div className="mt-4 pt-4 border-t border-slate-50">
                      <select 
                        className="w-full bg-slate-50 border border-slate-200 rounded-xl px-2 py-2 text-[10px] font-bold focus:ring-2 focus:ring-blue-500 transition-all outline-none"
                        onChange={(e) => handleShiftAssign(day, e.target.value || null)}
                        value={roster?.shift_id || ""}
                      >
                        <option value="">Gán ca...</option>
                        {shifts?.map(s => (
                          <option key={s.id} value={s.id}>{s.name} ({s.start_time.substring(0,5)})</option>
                        ))}
                        <option value="">Nghỉ</option>
                      </select>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function X({ size, className }: { size: number; className?: string }) {
  return (
    <svg 
      xmlns="http://www.w3.org/2000/svg" 
      width={size} 
      height={size} 
      viewBox="0 0 24 24" 
      fill="none" 
      stroke="currentColor" 
      strokeWidth="3" 
      strokeLinecap="round" 
      strokeLinejoin="round" 
      className={className}
    >
      <path d="M18 6 6 18"/><path d="m6 6 12 12"/>
    </svg>
  );
}
