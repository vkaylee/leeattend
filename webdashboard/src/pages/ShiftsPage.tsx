import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  CalendarRange, 
  Plus, 
  Clock, 
  Calendar, 
  User, 
  Trash2, 
  Save,
  ChevronRight,
  ChevronLeft,
  Info,
  X,
  LayoutGrid,
  List
} from "lucide-react";
import { 
  fetchEmployees, 
  fetchShifts, 
  assignShift, 
  createShift,
  fetchEmployeeRosters,
  upsertRoster,
  getCurrentWorkspaceId,
  Employee,
  Shift
} from "../lib/api";
import { useToast } from "../components/Toast";

type TabType = "assignments" | "weekly-roster" | "directory";

export default function ShiftsPage() {
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  const [activeTab, setActiveTab] = useState<TabType>("weekly-roster");
  
  // Selection states
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  
  // Weekly Roster Navigation
  const [currentDate, setCurrentDate] = useState(new Date());

  // Fixed Assignment Form State
  const [selectedShiftId, setSelectedShiftId] = useState("");
  const [startDate, setStartDate] = useState(new Date().toISOString().split('T')[0]);
  const [endDate, setEndDate] = useState<string>("");
  const [isForever, setIsForever] = useState(true);
  
  // Create Shift Modal State
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [newShiftName, setNewShiftName] = useState("");
  const [newShiftStart, setNewShiftStart] = useState("08:00");
  const [newShiftEnd, setNewShiftEnd] = useState("17:00");
  const [newShiftBreak, setNewShiftBreak] = useState(60);

  // Queries
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

  // Weekly Dates Logic
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
    enabled: !!selectedEmployee && !!wsId && activeTab === "weekly-roster",
  });

  // Mutations
  const assignMutation = useMutation({
    mutationFn: (data: any) => assignShift(wsId, data),
    onSuccess: () => {
      toast.success("Đã gán ca làm việc cho nhân viên thành công.");
      setSelectedShiftId("");
      setEndDate("");
      setIsForever(true);
      queryClient.invalidateQueries({ queryKey: ["employee_shifts"] });
    },
    onError: (error: any) => toast.error(error.message, "Lỗi gán ca")
  });

  const rosterMutation = useMutation({
    mutationFn: (data: any) => upsertRoster(wsId, selectedEmployee!.id, data),
    onSuccess: () => {
      toast.success("Đã cập nhật lịch trực.");
      queryClient.invalidateQueries({ queryKey: ["rosters", selectedEmployee?.id] });
    },
    onError: (error: any) => toast.error(error.message, "Lỗi cập nhật lịch")
  });

  const createShiftMutation = useMutation({
    mutationFn: (data: any) => createShift(wsId, data),
    onSuccess: () => {
      toast.success(`Loại ca "${newShiftName}" đã được khởi tạo.`);
      setIsCreateModalOpen(false);
      setNewShiftName("");
      setNewShiftStart("08:00");
      setNewShiftEnd("17:00");
      setNewShiftBreak(60);
      queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
    },
    onError: (error: any) => toast.error(error.message, "Lỗi tạo ca")
  });

  // Handlers
  const handleAssign = () => {
    if (!selectedEmployee || !selectedShiftId || !startDate) {
      toast.warning("Vui lòng chọn đầy đủ nhân viên, ca làm và ngày bắt đầu.");
      return;
    }
    assignMutation.mutate({
      employee_id: selectedEmployee.id,
      shift_id: selectedShiftId,
      start_date: startDate,
      end_date: isForever ? null : endDate,
    });
  };

  const handleShiftAssign = (date: Date, shiftId: string | null) => {
    if (!selectedEmployee) return;
    rosterMutation.mutate({
      employee_id: selectedEmployee.id,
      date: date.toISOString().split('T')[0],
      shift_id: shiftId,
    });
  };

  const handleCreateShift = () => {
    if (!newShiftName || !newShiftStart || !newShiftEnd) {
      toast.warning("Tên ca và thời gian làm việc không được để trống.");
      return;
    }
    createShiftMutation.mutate({
      name: newShiftName,
      start_time: newShiftStart + ":00",
      end_time: newShiftEnd + ":00",
      break_duration_minutes: newShiftBreak,
      break_is_overtime: false,
      early_punch_in_grace_period_minutes: 0,
    });
  };

  return (
    <div className="space-y-6">
      {/* Header section */}
      <div className="flex flex-col md:flex-row justify-between items-start md:items-end gap-4">
        <div>
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">Hệ thống Lịch trình</h1>
          <p className="text-slate-500 mt-1">Quản lý ca làm việc linh hoạt và cố định</p>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          <div className="bg-slate-100 p-1 rounded-2xl flex border border-slate-200 shadow-inner">
            <button 
              onClick={() => setActiveTab("weekly-roster")}
              className={`px-4 py-2 rounded-xl text-[10px] font-black tracking-widest transition-all flex items-center gap-2 ${activeTab === 'weekly-roster' ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
            >
              <LayoutGrid size={14} /> LỊCH TUẦN
            </button>
            <button 
              onClick={() => setActiveTab("assignments")}
              className={`px-4 py-2 rounded-xl text-[10px] font-black tracking-widest transition-all flex items-center gap-2 ${activeTab === 'assignments' ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
            >
              <Calendar size={14} /> GÁN CỐ ĐỊNH
            </button>
            <button 
              onClick={() => setActiveTab("directory")}
              className={`px-4 py-2 rounded-xl text-[10px] font-black tracking-widest transition-all flex items-center gap-2 ${activeTab === 'directory' ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
            >
              <List size={14} /> DANH MỤC
            </button>
          </div>
          <button 
            onClick={() => setIsCreateModalOpen(true)}
            className="bg-blue-600 hover:bg-blue-700 text-white font-black px-6 py-2.5 rounded-2xl flex items-center gap-2 shadow-lg shadow-blue-100 transition-all active:scale-95"
          >
            <Plus className="w-5 h-5" />
            <span className="text-xs uppercase tracking-widest">Tạo ca</span>
          </button>
        </div>
      </div>

      {activeTab !== "directory" ? (
        <div className="grid grid-cols-12 gap-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
          {/* Sidebar: Employee List */}
          <div className="col-span-12 lg:col-span-4 xl:col-span-3 bg-white border border-slate-200 rounded-[32px] overflow-hidden shadow-sm flex flex-col h-fit lg:max-h-[700px]">
            <div className="p-5 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
              <span className="font-black text-[10px] uppercase tracking-widest text-slate-400">Nhân sự</span>
              <span className="bg-white px-2 py-1 rounded-lg text-[10px] font-black text-slate-400 border border-slate-200">
                {employees?.length || 0} NGƯỜI
              </span>
            </div>
            <div className="flex-1 overflow-y-auto divide-y divide-slate-50">
              {employees?.map(emp => (
                <button
                  key={emp.id}
                  onClick={() => setSelectedEmployee(emp)}
                  className={`w-full flex items-center gap-4 p-4 hover:bg-slate-50 transition-all text-left group ${selectedEmployee?.id === emp.id ? 'bg-blue-50/50' : ''}`}
                >
                  <div className={`w-10 h-10 rounded-2xl flex items-center justify-center font-bold text-base transition-transform group-active:scale-95 ${selectedEmployee?.id === emp.id ? 'bg-blue-600 text-white shadow-lg shadow-blue-100' : 'bg-slate-100 text-slate-400'}`}>
                    {emp.last_name[0]}
                  </div>
                  <div className="flex-1">
                    <div className={`font-bold text-sm leading-tight ${selectedEmployee?.id === emp.id ? 'text-blue-700' : 'text-slate-900'}`}>
                      {emp.first_name} {emp.last_name}
                    </div>
                    <div className="text-[10px] text-slate-400 font-black tracking-tighter uppercase mt-0.5">{emp.employee_id_number}</div>
                  </div>
                  <ChevronRight className={`w-4 h-4 transition-transform ${selectedEmployee?.id === emp.id ? 'text-blue-400 translate-x-1' : 'text-slate-200'}`} />
                </button>
              ))}
            </div>
          </div>

          {/* Main Content: Tabs View */}
          <div className="col-span-12 lg:col-span-8 xl:col-span-9 space-y-6">
            {!selectedEmployee ? (
              <div className="bg-white border border-slate-200 rounded-[40px] h-[500px] flex flex-col items-center justify-center text-center p-12">
                <div className="bg-slate-50 p-6 rounded-[32px] mb-6">
                  <User className="w-12 h-12 text-slate-300" />
                </div>
                <h3 className="text-xl font-bold text-slate-900">Chọn nhân sự để tiếp tục</h3>
                <p className="text-slate-500 mt-2 max-w-xs mx-auto text-sm">
                  Vui lòng chọn một nhân viên từ danh sách bên trái để bắt đầu sắp xếp lịch làm việc.
                </p>
              </div>
            ) : activeTab === "weekly-roster" ? (
              /* TAB: WEEKLY ROSTER */
              <div className="space-y-4">
                <div className="flex items-center justify-between bg-white p-4 rounded-[32px] border border-slate-200 shadow-sm">
                  <div className="flex items-center gap-4">
                    <div className="w-12 h-12 bg-blue-600 rounded-2xl flex items-center justify-center text-white text-lg font-black">
                      {selectedEmployee.last_name[0]}
                    </div>
                    <div>
                      <h2 className="text-lg font-bold text-slate-900 leading-tight">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
                      <p className="text-[10px] font-black uppercase tracking-widest text-slate-400">Lịch trực linh hoạt theo tuần</p>
                    </div>
                  </div>
                  
                  <div className="flex items-center gap-3">
                    <button 
                      onClick={() => {
                        const d = new Date(currentDate);
                        d.setDate(d.getDate() - 7);
                        setCurrentDate(d);
                      }}
                      className="p-2.5 hover:bg-slate-100 border border-slate-100 rounded-xl transition-all"
                    >
                      <ChevronLeft size={18} />
                    </button>
                    <div className="px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl text-xs font-bold text-slate-600">
                      TUẦN: {weekDays[0].toLocaleDateString('vi-VN')} - {weekDays[6].toLocaleDateString('vi-VN')}
                    </div>
                    <button 
                      onClick={() => {
                        const d = new Date(currentDate);
                        d.setDate(d.getDate() + 7);
                        setCurrentDate(d);
                      }}
                      className="p-2.5 hover:bg-slate-100 border border-slate-100 rounded-xl transition-all"
                    >
                      <ChevronRight size={18} />
                    </button>
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-7 gap-4">
                  {weekDays.map((day, idx) => {
                    const dateStr = day.toISOString().split('T')[0];
                    const roster = rosters?.find(r => r.date === dateStr);
                    const assignedShift = shifts?.find(s => s.id === roster?.shift_id);
                    const isToday = day.toDateString() === new Date().toDateString();
                    
                    return (
                      <div key={idx} className={`bg-white border p-4 rounded-[32px] flex flex-col min-h-[350px] transition-all relative group ${isToday ? 'border-blue-300 ring-4 ring-blue-50 shadow-lg' : 'border-slate-100 hover:border-slate-200 shadow-sm'}`}>
                        <div className="text-center mb-5 pb-5 border-b border-slate-50">
                          <div className={`text-[10px] font-black uppercase tracking-tighter ${isToday ? 'text-blue-500' : 'text-slate-400'}`}>
                            {day.toLocaleDateString('vi-VN', { weekday: 'short' })}
                          </div>
                          <div className={`text-2xl font-black mt-1 ${isToday ? 'text-blue-600' : 'text-slate-900'}`}>
                            {day.getDate()}
                          </div>
                        </div>

                        <div className="flex-1 space-y-4">
                          <div className="text-[9px] font-black text-slate-300 uppercase tracking-widest flex items-center gap-1">
                            <Clock size={10} /> Ca làm việc
                          </div>
                          
                          {assignedShift ? (
                            <div className="bg-blue-50 border border-blue-100 p-4 rounded-[24px] relative group/item animate-in zoom-in-95">
                              <div className="font-black text-blue-700 text-xs mb-1 uppercase tracking-tight leading-tight">{assignedShift.name}</div>
                              <div className="flex items-center gap-1.5 text-[10px] text-blue-500 font-bold">
                                {assignedShift.start_time.substring(0, 5)} - {assignedShift.end_time.substring(0, 5)}
                              </div>
                              <button 
                                onClick={() => handleShiftAssign(day, null)}
                                className="absolute -top-2 -right-2 bg-white border border-slate-200 rounded-full p-1.5 shadow-md opacity-0 group-hover/item:opacity-100 transition-opacity hover:bg-rose-50 hover:border-rose-200 group/btn"
                              >
                                <X size={12} className="text-slate-400 group-hover/btn:text-rose-500" />
                              </button>
                            </div>
                          ) : (
                            <div className="bg-slate-50/50 border border-slate-100 border-dashed p-6 rounded-[24px] text-[10px] text-slate-400 font-black text-center uppercase tracking-widest flex flex-col items-center justify-center gap-2">
                              NGHỈ
                            </div>
                          )}
                        </div>

                        <div className="mt-5 pt-5 border-t border-slate-50">
                          <select 
                            className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-3 py-3 text-[10px] font-black uppercase tracking-tight focus:ring-2 focus:ring-blue-500 transition-all outline-none text-slate-600 appearance-none text-center"
                            onChange={(e) => handleShiftAssign(day, e.target.value || null)}
                            value={roster?.shift_id || ""}
                          >
                            <option value="">Gán ca...</option>
                            <optgroup label="DANH MỤC CA">
                              {shifts?.map(s => (
                                <option key={s.id} value={s.id}>{s.name} ({s.start_time.substring(0,5)})</option>
                              ))}
                            </optgroup>
                            <option value="">Xóa ca (Nghỉ)</option>
                          </select>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            ) : (
              /* TAB: ASSIGNMENTS (Fixed/Temporal) */
              <div className="bg-white border border-slate-200 rounded-[40px] p-8 shadow-sm animate-in fade-in zoom-in-95 duration-300">
                <div className="flex items-center gap-4 mb-10 pb-10 border-b border-slate-50">
                  <div className="w-20 h-20 bg-blue-600 rounded-[28px] flex items-center justify-center text-white text-3xl font-black shadow-xl shadow-blue-100">
                    {selectedEmployee.last_name[0]}
                  </div>
                  <div>
                    <h2 className="text-2xl font-bold text-slate-900">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
                    <p className="text-[10px] font-black uppercase tracking-widest text-slate-400 mt-1">Gán lịch trình cố định hoặc dài hạn</p>
                  </div>
                </div>

                <div className="space-y-10">
                  <div className="space-y-4">
                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest flex items-center gap-2 ml-1">
                      <Clock size={12} /> Chọn Ca Làm Việc Mặc Định
                    </label>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      {shifts?.map(shift => (
                        <button
                          key={shift.id}
                          onClick={() => setSelectedShiftId(shift.id)}
                          className={`p-5 rounded-[28px] border-2 text-left transition-all relative overflow-hidden group ${selectedShiftId === shift.id ? 'border-blue-600 bg-blue-50/50' : 'border-slate-100 hover:border-slate-200 bg-slate-50/30'}`}
                        >
                          <div className={`font-black text-sm uppercase tracking-tight ${selectedShiftId === shift.id ? 'text-blue-700' : 'text-slate-700'}`}>{shift.name}</div>
                          <div className="text-[10px] text-slate-400 font-bold mt-1.5 flex items-center gap-1">
                            <Clock size={10} /> {shift.start_time.substring(0, 5)} - {shift.end_time.substring(0, 5)}
                          </div>
                          {selectedShiftId === shift.id && (
                            <div className="absolute top-4 right-4 w-3 h-3 bg-blue-600 rounded-full animate-pulse" />
                          )}
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                    <div className="space-y-3">
                      <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest flex items-center gap-2 ml-1">
                        <Calendar size={12} /> Hiệu lực từ ngày
                      </label>
                      <input 
                        type="date" 
                        value={startDate}
                        onChange={(e) => setStartDate(e.target.value)}
                        className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                      />
                    </div>

                    <div className="space-y-3">
                      <div className="flex justify-between items-center px-1">
                        <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest flex items-center gap-2">
                          <Calendar size={12} /> Ngày Kết Thúc
                        </label>
                        <button 
                          onClick={() => setIsForever(!isForever)}
                          className={`text-[9px] font-black px-3 py-1 rounded-full border transition-all uppercase tracking-tighter ${isForever ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-100 text-slate-400 border-slate-200'}`}
                        >
                          {isForever ? "VÔ THỜI HẠN" : "CÓ THỜI HẠN"}
                        </button>
                      </div>
                      <input 
                        type="date" 
                        disabled={isForever}
                        value={endDate}
                        onChange={(e) => setEndDate(e.target.value)}
                        className={`w-full border rounded-2xl px-5 py-4 font-bold transition-all outline-none ${isForever ? 'bg-slate-100 text-slate-300 border-slate-50' : 'bg-slate-50 text-slate-700 border-slate-200 focus:ring-2 focus:ring-blue-500'}`}
                      />
                    </div>
                  </div>

                  <button
                    onClick={handleAssign}
                    disabled={assignMutation.isPending}
                    className="w-full bg-slate-900 text-white font-black py-5 rounded-[28px] flex items-center justify-center gap-3 hover:bg-black transition-all active:scale-[0.98] disabled:opacity-50 shadow-2xl shadow-slate-200 uppercase tracking-widest text-xs"
                  >
                    <Save size={18} /> Gán ca làm việc
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      ) : (
        /* TAB: DIRECTORY */
        <div className="bg-white border border-slate-200 rounded-[40px] overflow-hidden shadow-sm animate-in fade-in slide-in-from-bottom-4 duration-500">
          <div className="p-6 border-b border-slate-100 bg-slate-50/50 flex items-center justify-between">
            <h2 className="font-black text-[10px] text-slate-400 uppercase tracking-widest">Danh mục các loại ca</h2>
            <span className="text-[10px] font-bold text-slate-400 italic">Tổng số: {shifts?.length || 0}</span>
          </div>
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50/20">
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100">Tên loại ca</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100">Khung giờ</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100">Nghỉ giữa ca</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100 text-right">Hành động</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {shifts?.map(shift => (
                <tr key={shift.id} className="hover:bg-slate-50/50 transition-colors group">
                  <td className="px-8 py-6">
                    <div className="font-bold text-slate-900 text-lg">{shift.name}</div>
                    <div className="text-[10px] text-slate-400 font-black tracking-widest uppercase mt-1">{shift.id}</div>
                  </td>
                  <td className="px-8 py-6">
                    <div className="flex items-center gap-3">
                      <div className="bg-blue-50 text-blue-700 px-4 py-2 rounded-xl font-mono font-black text-sm border border-blue-100">
                        {shift.start_time.substring(0, 5)}
                      </div>
                      <ChevronRight className="w-4 h-4 text-slate-200" />
                      <div className="bg-blue-50 text-blue-700 px-4 py-2 rounded-xl font-mono font-black text-sm border border-blue-100">
                        {shift.end_time.substring(0, 5)}
                      </div>
                    </div>
                  </td>
                  <td className="px-8 py-6">
                    <div className="flex items-center gap-2 font-bold text-sm text-slate-600">
                      <Clock className="w-4 h-4 text-slate-300" />
                      {shift.break_duration_minutes} phút
                    </div>
                  </td>
                  <td className="px-8 py-6 text-right">
                    <button className="p-3 bg-slate-50 text-slate-400 hover:text-rose-600 hover:bg-rose-50 rounded-2xl transition-all border border-transparent hover:border-rose-100 opacity-0 group-hover:opacity-100">
                      <Trash2 className="w-5 h-5" />
                    </button>
                  </td>
                </tr>
              ))}
              {shifts?.length === 0 && (
                <tr>
                  <td colSpan={4} className="py-32 text-center">
                    <div className="bg-slate-50 w-20 h-20 rounded-[32px] flex items-center justify-center mx-auto mb-6">
                      <Clock size={32} className="text-slate-200" />
                    </div>
                    <p className="text-slate-300 font-black uppercase tracking-widest text-xs">Hệ thống chưa có loại ca nào.</p>
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      {/* Modal: Create Shift */}
      {isCreateModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/40 backdrop-blur-sm animate-in fade-in duration-200">
          <div className="bg-white rounded-[44px] shadow-2xl w-full max-w-md overflow-hidden animate-in zoom-in-95 duration-200 border border-slate-100">
            <div className="p-10 border-b border-slate-50 flex justify-between items-center bg-slate-50/30">
              <div>
                <h3 className="text-2xl font-black text-slate-900 uppercase tracking-tight leading-tight">Thiết lập ca</h3>
                <p className="text-[10px] text-slate-400 font-black mt-1 uppercase tracking-widest">KHAI BÁO THAM SỐ CA LÀM VIỆC</p>
              </div>
              <button onClick={() => setIsCreateModalOpen(false)} className="p-3 hover:bg-white border border-transparent hover:border-slate-100 rounded-2xl transition-all">
                <X className="w-6 h-6 text-slate-300" />
              </button>
            </div>
            
            <div className="p-10 space-y-8">
              <div className="space-y-3">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Tên loại ca</label>
                <input 
                  type="text" 
                  placeholder="Hành chính, Ca sáng, Ca gãy..."
                  value={newShiftName}
                  onChange={(e) => setNewShiftName(e.target.value)}
                  className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all placeholder:text-slate-300"
                />
              </div>

              <div className="grid grid-cols-2 gap-5">
                <div className="space-y-3">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Giờ vào</label>
                  <input 
                    type="time" 
                    value={newShiftStart}
                    onChange={(e) => setNewShiftStart(e.target.value)}
                    className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                  />
                </div>
                <div className="space-y-3">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Giờ ra</label>
                  <input 
                    type="time" 
                    value={newShiftEnd}
                    onChange={(e) => setNewShiftEnd(e.target.value)}
                    className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                  />
                </div>
              </div>

              <div className="space-y-4">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1 flex justify-between">
                  <span>Giải lao</span>
                  <span className="text-blue-600 font-black tracking-tighter">{newShiftBreak} PHÚT</span>
                </label>
                <input 
                  type="range" 
                  min="0" 
                  max="120" 
                  step="15"
                  value={newShiftBreak}
                  onChange={(e) => setNewShiftBreak(parseInt(e.target.value))}
                  className="w-full h-2 bg-slate-100 rounded-full appearance-none cursor-pointer accent-blue-600 shadow-inner"
                />
              </div>

              <button
                onClick={handleCreateShift}
                disabled={createShiftMutation.isPending}
                className="w-full bg-blue-600 text-white font-black py-5 rounded-[24px] flex items-center justify-center gap-3 hover:bg-blue-700 transition-all active:scale-95 shadow-xl shadow-blue-100 disabled:opacity-50 mt-4 uppercase tracking-widest text-xs"
              >
                <Save className="w-5 h-5" />
                Lưu loại ca
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}