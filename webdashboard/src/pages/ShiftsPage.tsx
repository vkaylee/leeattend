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
  List,
  History,
  ShieldCheck,
  Zap,
  Building2
} from "lucide-react";
import { 
  fetchEmployees, 
  fetchShifts, 
  assignShift, 
  createShift,
  updateShift,
  fetchWorkspace,
  fetchWorkspaceSchedules,
  fetchAssignmentHistory,
  upsertRoster,
  bulkUpsertRoster,
  deleteRoster,
  getCurrentWorkspaceId,
  Employee,
  Shift
} from "../lib/api";
import { useToast } from "../components/Toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { format, addDays } from "date-fns";
import { vi } from "date-fns/locale";
import { Combobox } from "@/components/ui/Combobox";
import { MousePointer2, Copy, Trash, Calendar as CalendarIcon } from "lucide-react";

type TabType = "assignments" | "weekly-roster" | "directory";

export default function ShiftsPage() {
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatDate, formatTime, formatDateTime } = useDateTimeFormatter();
  const [activeTab, setActiveTab] = useState<TabType>("weekly-roster");
  
  // Selection states
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [selectedDays, setSelectedDays] = useState<string[]>([]);
  
  // Shift Picker Modal State
  const [isShiftPickerOpen, setIsShiftPickerOpen] = useState(false);
  const [targetDates, setTargetDates] = useState<string[]>([]);
  
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
  const [newBreakStart, setNewBreakStart] = useState("");
  const [newBreakEnd, setNewBreakEnd] = useState("");
  const [isFixedBreak, setIsFixedBreak] = useState(false);
  const [isAutoDeduct, setIsAutoDeduct] = useState(true);

  // Edit Shift State
  const [editingShift, setEditingShift] = useState<Shift | null>(null);
  const [isEditCorrection, setIsEditCorrection] = useState(false);
  const [editEffectiveDate, setEditEffectiveDate] = useState(new Date().toISOString().split('T')[0]);

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

  const { data: workspace } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => fetchWorkspace(wsId),
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

  // Fetch Effective Schedules (Bulk)
  const { data: allSchedules } = useQuery({
    queryKey: ["schedules", wsId, startOfWeek.toISOString().split('T')[0]],
    queryFn: () => fetchWorkspaceSchedules(
      wsId,
      weekDays[0].toISOString().split('T')[0],
      weekDays[6].toISOString().split('T')[0]
    ),
    enabled: !!wsId && activeTab === "weekly-roster",
  });

  const { data: history } = useQuery({
    queryKey: ["assignment_history", wsId],
    queryFn: () => fetchAssignmentHistory(wsId),
    enabled: !!wsId,
  });

  // Find schedule for selected employee
  const employeeSchedule = allSchedules?.find(s => s.employee_id === selectedEmployee?.id);

  // Mutations
  const assignMutation = useMutation({
    mutationFn: (data: any) => assignShift(wsId, data),
    onSuccess: () => {
      toast.success("Đã gán ca làm việc cho nhân viên thành công.");
      setSelectedShiftId("");
      setEndDate("");
      setIsForever(true);
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: any) => toast.error(error.message, "Lỗi gán ca")
  });

  const rosterMutation = useMutation({
    mutationFn: (data: any) => upsertRoster(wsId, selectedEmployee!.id, data),
    onSuccess: () => {
      toast.success("Đã cập nhật lịch trực.");
      queryClient.invalidateQueries({ queryKey: ["schedules"] }); // Re-fetch effective schedules
      queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: any) => toast.error(error.message, "Lỗi cập nhật lịch")
  });

  const bulkRosterMutation = useMutation({
    mutationFn: (items: any[]) => bulkUpsertRoster(wsId, selectedEmployee!.id, items),
    onSuccess: () => {
      toast.success("Đã cập nhật lịch trực hàng loạt.");
      setSelectedDays([]);
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: any) => toast.error(error.message, "Lỗi cập nhật hàng loạt")
  });

  const deleteRosterMutation = useMutation({
    mutationFn: (date: string) => deleteRoster(wsId, selectedEmployee!.id, date),
    onSuccess: () => {
      toast.success("Đã gỡ bỏ ghi đè, quay về lịch mặc định.");
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: any) => toast.error(error.message, "Lỗi khi gỡ bỏ lịch")
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
      setNewBreakStart("");
      setNewBreakEnd("");
      setIsFixedBreak(false);
      setIsAutoDeduct(true);
      queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
    },
    onError: (error: any) => toast.error(error.message, "Lỗi tạo ca")
  });

  const updateShiftMutation = useMutation({
    mutationFn: ({ id, data }: { id: string, data: any }) => updateShift(wsId, id, data),
    onSuccess: () => {
      toast.success("Đã cập nhật cấu hình ca làm việc.");
      setEditingShift(null);
      queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: any) => toast.error(error.message, "Lỗi cập nhật ca")
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

  const handleBulkAssign = (shiftId: string | null) => {
    if (!selectedEmployee || selectedDays.length === 0) return;
    
    const items = selectedDays.map(date => ({
      date,
      shift_id: shiftId,
    }));
    
    bulkRosterMutation.mutate(items);
  };

  const handleAssignAllWeek = (shiftId: string | null) => {
    if (!selectedEmployee) return;
    
    const items = weekDays.map(day => ({
      date: day.toISOString().split('T')[0],
      shift_id: shiftId,
    }));
    
    bulkRosterMutation.mutate(items);
  };

  const handleCopyPreviousWeek = async () => {
    if (!selectedEmployee || !allSchedules) return;
    
    const prevWeekStart = addDays(startOfWeek, -7);
    const prevWeekEnd = addDays(startOfWeek, -1);
    
    try {
      const prevSchedules = await fetchWorkspaceSchedules(
        wsId, 
        prevWeekStart.toISOString().split('T')[0], 
        prevWeekEnd.toISOString().split('T')[0]
      );
      
      const empPrev = prevSchedules.find(s => s.employee_id === selectedEmployee.id);
      if (!empPrev) {
        toast.warning("Không tìm thấy dữ liệu tuần trước.");
        return;
      }

      const items = empPrev.schedules.map((s, idx) => ({
        date: weekDays[idx].toISOString().split('T')[0],
        shift_id: s.shift_id,
      }));

      bulkRosterMutation.mutate(items);
    } catch (e: any) {
      toast.error(e.message, "Lỗi sao chép");
    }
  };

  const handleCreateShift = () => {
    if (!newShiftName || !newShiftStart || !newShiftEnd) {
      toast.warning("Tên ca và thời gian làm việc không được để trống.");
      return;
    }

    if (isFixedBreak && (!newBreakStart || !newBreakEnd)) {
      toast.warning("Vui lòng nhập giờ bắt đầu và kết thúc nghỉ trưa.");
      return;
    }
    
    // Calculate duration if fixed break is selected
    let duration = newShiftBreak;
    if (isFixedBreak && newBreakStart && newBreakEnd) {
      const start = new Date(`2000-01-01T${newBreakStart}`);
      const end = new Date(`2000-01-01T${newBreakEnd}`);
      const diff = (end.getTime() - start.getTime()) / (1000 * 60);
      if (diff > 0) duration = diff;
    }

    createShiftMutation.mutate({
      name: newShiftName,
      start_time: newShiftStart + ":00",
      end_time: newShiftEnd + ":00",
      break_duration_minutes: duration,
      break_start: (isFixedBreak && newBreakStart) ? newBreakStart + ":00" : undefined,
      break_end: (isFixedBreak && newBreakEnd) ? newBreakEnd + ":00" : undefined,
      is_break_auto_deduct: isAutoDeduct,
      break_is_overtime: false,
      early_punch_in_grace_period_minutes: 0,
    });
  };

  const handleUpdateShift = () => {
    if (!editingShift) return;

    updateShiftMutation.mutate({
      id: editingShift.id,
      data: {
        name: newShiftName,
        start_time: newShiftStart.length === 5 ? newShiftStart + ":00" : newShiftStart,
        end_time: newShiftEnd.length === 5 ? newShiftEnd + ":00" : newShiftEnd,
        break_duration_minutes: newShiftBreak,
        break_start: (isFixedBreak && newBreakStart) ? (newBreakStart.length === 5 ? newBreakStart + ":00" : newBreakStart) : null,
        break_end: (isFixedBreak && newBreakEnd) ? (newBreakEnd.length === 5 ? newBreakEnd + ":00" : newBreakEnd) : null,
        is_break_auto_deduct: isAutoDeduct,
        break_is_overtime: false,
        early_punch_in_grace_period_minutes: 0,
        is_correction: isEditCorrection,
        effective_date: isEditCorrection ? undefined : editEffectiveDate
      }
    });
  };

  const openEditModal = (shift: Shift) => {
    setEditingShift(shift);
    setNewShiftName(shift.name);
    setNewShiftStart(shift.start_time.substring(0, 5));
    setNewShiftEnd(shift.end_time.substring(0, 5));
    setNewShiftBreak(shift.break_duration_minutes);
    setNewBreakStart(shift.break_start?.substring(0, 5) || "");
    setNewBreakEnd(shift.break_end?.substring(0, 5) || "");
    setIsFixedBreak(!!shift.break_start);
    setIsAutoDeduct(shift.is_break_auto_deduct ?? true);
    setIsEditCorrection(false);
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
          <div className="col-span-12 lg:col-span-8 xl:col-span-9 space-y-8">
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
              <>
                <div className="space-y-4">
                  <div className="flex items-center justify-between bg-white p-4 rounded-[32px] border border-slate-200 shadow-sm">
                    <div className="flex items-center gap-4">
                      <div className="w-12 h-12 bg-blue-600 rounded-2xl flex items-center justify-center text-white text-lg font-black">
                        {selectedEmployee.last_name[0]}
                      </div>
                      <div>
                        <h2 className="text-lg font-bold text-slate-900 leading-tight">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
                        <p className="text-[10px] font-black uppercase tracking-widest text-slate-400">Lịch trực tuần hiện tại</p>
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-3">
                      <div className="w-56">
                        <Combobox
                          options={[
                            { value: "copy-prev", label: "Sao chép tuần trước", icon: <Copy className="w-4 h-4" /> },
                            { value: "assign-all", label: "Gán ca cả tuần này", icon: <CalendarIcon className="w-4 h-4" /> },
                            { value: "clear-week", label: "Xóa lịch tuần này", icon: <Trash className="w-4 h-4" /> },
                          ]}
                          onChange={(val) => {
                            if (val === "copy-prev") {
                              handleCopyPreviousWeek();
                            } else if (val === "clear-week") {
                              handleAssignAllWeek(null);
                            } else if (val === "assign-all") {
                              setTargetDates(weekDays.map(d => d.toISOString().split('T')[0]));
                              setIsShiftPickerOpen(true);
                            }
                          }}
                          placeholder="Thao tác nhanh..."
                          className="h-10 bg-slate-100 border-slate-200 text-[10px] py-2 px-3 rounded-xl uppercase tracking-tighter"
                        />
                      </div>

                      <button 
                        onClick={() => {
                          const d = new Date(currentDate);
                          d.setDate(d.getDate() - 7);
                          setCurrentDate(d);
                          setSelectedDays([]);
                        }}
                        className="p-2.5 hover:bg-slate-100 border border-slate-100 rounded-xl transition-all"
                      >
                        <ChevronLeft size={18} />
                      </button>
                      <div className="px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl text-xs font-bold text-slate-600">
                        TUẦN: {formatDate(weekDays[0])} - {formatDate(weekDays[6])}
                      </div>
                      <button 
                        onClick={() => {
                          const d = new Date(currentDate);
                          d.setDate(d.getDate() + 7);
                          setCurrentDate(d);
                          setSelectedDays([]);
                        }}
                        className="p-2.5 hover:bg-slate-100 border border-slate-100 rounded-xl transition-all"
                      >
                        <ChevronRight size={18} />
                      </button>
                    </div>
                  </div>

                  {selectedDays.length > 0 && (
                    <div className="bg-blue-600 text-white p-4 rounded-[24px] flex items-center justify-between shadow-lg shadow-blue-100 animate-in slide-in-from-top-4">
                      <div className="flex items-center gap-4">
                        <span className="text-xs font-black uppercase tracking-widest">Đã chọn {selectedDays.length} ngày</span>
                        <div className="h-4 w-px bg-white/20" />
                        <button 
                          onClick={() => {
                            setTargetDates(selectedDays);
                            setIsShiftPickerOpen(true);
                          }}
                          className="bg-white/10 border border-white/20 rounded-xl px-4 py-2 text-[10px] font-black uppercase tracking-widest hover:bg-white/20 transition-all flex items-center gap-2"
                        >
                          <Plus size={14} /> Gán ca cho mục đã chọn
                        </button>
                      </div>
                      <button onClick={() => setSelectedDays([])} className="hover:bg-white/10 p-2 rounded-xl transition-all">
                        <X size={18} />
                      </button>
                    </div>
                  )}

                  <div className="grid grid-cols-1 md:grid-cols-7 gap-4">
                    {weekDays.map((day, idx) => {
                      const dateStr = day.toISOString().split('T')[0];
                      const isSelected = selectedDays.includes(dateStr);
                      const effectiveSchedule = employeeSchedule?.schedules.find(s => s.date === dateStr);
                      const isToday = day.toDateString() === new Date().toDateString();
                      
                      const isRoster = effectiveSchedule?.source === "roster";
                      const isFixed = effectiveSchedule?.source === "fixed";
                      const isDept = effectiveSchedule?.source === "department";

                      return (
                        <div 
                          key={idx} 
                          onClick={() => {
                            if (selectedDays.includes(dateStr)) {
                              setSelectedDays(selectedDays.filter(d => d !== dateStr));
                            } else {
                              setSelectedDays([...selectedDays, dateStr]);
                            }
                          }}
                          className={`bg-white border p-4 rounded-[32px] flex flex-col min-h-[350px] transition-all relative group cursor-pointer ${isSelected ? 'border-blue-500 ring-4 ring-blue-50 shadow-md' : isToday ? 'border-blue-300 ring-4 ring-blue-50 shadow-lg' : 'border-slate-100 hover:border-slate-200 shadow-sm'}`}
                        >
                          <div className="text-center mb-5 pb-5 border-b border-slate-50 relative">
                            {isSelected && (
                              <div className="absolute -top-1 -right-1 bg-blue-600 text-white rounded-full p-1 shadow-sm">
                                <ShieldCheck size={12} />
                              </div>
                            )}
                            <div className={`text-[10px] font-black uppercase tracking-tighter ${isToday ? 'text-blue-500' : 'text-slate-400'}`}>
                              {format(day, "EEEE", { locale: vi })}
                            </div>
                            <div className={`text-2xl font-black mt-1 ${isToday ? 'text-blue-600' : 'text-slate-900'}`}>
                              {day.getDate()}
                            </div>
                          </div>

                          <div className="flex-1 space-y-4">
                            {/* Source Badge */}
                            <div className="flex justify-center">
                              {isRoster && <span className="text-[9px] font-black px-2 py-0.5 rounded-full bg-purple-50 text-purple-600 border border-purple-100 uppercase tracking-tighter flex items-center gap-1"><Zap size={8} /> Linh hoạt</span>}
                              {isFixed && <span className="text-[9px] font-black px-2 py-0.5 rounded-full bg-emerald-50 text-emerald-600 border border-emerald-100 uppercase tracking-tighter flex items-center gap-1"><ShieldCheck size={8} /> Cố định</span>}
                              {isDept && <span className="text-[9px] font-black px-2 py-0.5 rounded-full bg-slate-50 text-slate-500 border border-slate-100 uppercase tracking-tighter flex items-center gap-1"><Building2 size={8} /> Phòng ban</span>}
                            </div>
                            
                            {effectiveSchedule?.shift_id ? (
                              <div className={`p-4 rounded-[24px] relative group/item animate-in zoom-in-95 border ${isRoster ? 'bg-purple-50/50 border-purple-100' : 'bg-blue-50 border-blue-100'}`}>
                                <div className={`font-black text-xs mb-1 uppercase tracking-tight leading-tight ${isRoster ? 'text-purple-700' : 'text-blue-700'}`}>
                                  {effectiveSchedule.shift_name}
                                </div>
                                <div className={`flex items-center gap-1.5 text-[10px] font-bold ${isRoster ? 'text-purple-500' : 'text-blue-500'}`}>
                                  {formatTime(effectiveSchedule.start_time)} - {formatTime(effectiveSchedule.end_time)}
                                </div>

                                {effectiveSchedule.break_duration_minutes !== null && (
                                  <div className="mt-2 pt-2 border-t border-black/5 flex items-center gap-1.5 text-[9px] font-black uppercase tracking-tighter text-slate-400">
                                    <Clock size={8} /> 
                                    {effectiveSchedule.break_start ? (
                                      <span>Nghỉ: {effectiveSchedule.break_start.substring(0, 5)} - {effectiveSchedule.break_end?.substring(0, 5)}</span>
                                    ) : (
                                      <span>Nghỉ: {effectiveSchedule.break_duration_minutes}m</span>
                                    )}
                                  </div>
                                )}
                                
                                {isRoster && (
                                  <button 
                                    onClick={(e) => {
                                      e.stopPropagation();
                                      deleteRosterMutation.mutate(dateStr);
                                    }}
                                    className="absolute -top-2 -right-2 bg-white border border-slate-200 rounded-full p-1.5 shadow-md opacity-0 group-hover/item:opacity-100 transition-opacity hover:bg-rose-50 hover:border-rose-200 group/btn"
                                  >
                                    <X size={12} className="text-slate-400 group-hover/btn:text-rose-500" />
                                  </button>
                                )}
                              </div>
                            ) : (
                              <div className="bg-slate-50/50 border border-slate-100 border-dashed p-6 rounded-[24px] text-[10px] text-slate-400 font-black text-center uppercase tracking-widest flex flex-col items-center justify-center gap-2">
                                NGHỈ
                              </div>
                            )}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>

                {/* History Section */}
                <div className="bg-white border border-slate-200 rounded-[40px] p-8 shadow-sm animate-in fade-in slide-in-from-bottom-8 duration-500">
                  <div className="flex items-center gap-3 mb-6">
                    <History className="w-5 h-5 text-slate-400" />
                    <h3 className="text-sm font-black text-slate-900 uppercase tracking-widest">Lịch sử thay đổi gán ca</h3>
                  </div>
                  
                  <div className="overflow-hidden rounded-3xl border border-slate-100">
                    <table className="w-full text-left">
                      <thead className="bg-slate-50/50">
                        <tr>
                          <th className="p-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">Thời gian</th>
                          <th className="p-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">Nhân viên</th>
                          <th className="p-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">Loại gán</th>
                          <th className="p-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">Ca làm việc</th>
                          <th className="p-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">Phạm vi</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-slate-50">
                        {history?.filter(h => h.employee_id === selectedEmployee.id || !selectedEmployee).map(h => (
                          <tr key={h.id} className="hover:bg-slate-50/50">
                            <td className="p-4 text-xs font-bold text-slate-500">
                              {formatDateTime(h.created_at)}
                            </td>
                            <td className="p-4 text-sm font-bold text-slate-900">{h.employee_name}</td>
                            <td className="p-4">
                              {h.assignment_type === 'roster' ? (
                                <span className="inline-flex items-center gap-1 px-2 py-1 rounded-lg bg-purple-50 text-purple-600 text-[10px] font-black uppercase tracking-tighter border border-purple-100">
                                  <Zap size={10} /> Linh hoạt
                                </span>
                              ) : (
                                <span className="inline-flex items-center gap-1 px-2 py-1 rounded-lg bg-emerald-50 text-emerald-600 text-[10px] font-black uppercase tracking-tighter border border-emerald-100">
                                  <ShieldCheck size={10} /> Cố định
                                </span>
                              )}
                            </td>
                            <td className="p-4 font-bold text-slate-700 text-xs">{h.shift_name || "NGHỈ"}</td>
                            <td className="p-4 font-mono text-xs text-slate-500">{h.date_range}</td>
                          </tr>
                        ))}
                        {(!history || history.length === 0) && (
                          <tr>
                            <td colSpan={5} className="p-8 text-center text-xs font-bold text-slate-300 uppercase tracking-widest">Chưa có dữ liệu lịch sử</td>
                          </tr>
                        )}
                      </tbody>
                    </table>
                  </div>
                </div>
              </>
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
                          <div className="flex flex-wrap gap-x-3 gap-y-1 mt-1.5">
                            <div className="text-[10px] text-slate-400 font-bold flex items-center gap-1">
                              <Clock size={10} /> {formatTime(shift.start_time)} - {formatTime(shift.end_time)}
                            </div>
                            <div className="text-[10px] text-slate-400 font-bold flex items-center gap-1">
                              <Info size={10} /> 
                              {shift.break_start ? (
                                <span>{shift.break_start.substring(0, 5)}-{shift.break_end?.substring(0, 5)}</span>
                              ) : (
                                <span>Nghỉ {shift.break_duration_minutes}p</span>
                              )}
                            </div>
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
                        {formatTime(shift.start_time)}
                      </div>
                      <ChevronRight className="w-4 h-4 text-slate-200" />
                      <div className="bg-blue-50 text-blue-700 px-4 py-2 rounded-xl font-mono font-black text-sm border border-blue-100">
                        {formatTime(shift.end_time)}
                      </div>
                    </div>
                  </td>
                  <td className="px-8 py-6">
                    {shift.break_start && shift.break_end ? (
                      <div>
                        <div className="flex items-center gap-2 font-bold text-sm text-slate-700">
                          <span className="text-xs bg-slate-100 px-2 py-0.5 rounded text-slate-500 font-mono">{shift.break_start.substring(0, 5)}</span>
                          <span className="text-slate-300">-</span>
                          <span className="text-xs bg-slate-100 px-2 py-0.5 rounded text-slate-500 font-mono">{shift.break_end.substring(0, 5)}</span>
                        </div>
                        {shift.is_break_auto_deduct && (
                          <div className="text-[9px] font-black text-blue-500 uppercase tracking-tight mt-1 flex items-center gap-1">
                            <Zap size={8} /> Auto-deduct
                          </div>
                        )}
                      </div>
                    ) : (
                      <div className="flex items-center gap-2 font-bold text-sm text-slate-600">
                        <Clock className="w-4 h-4 text-slate-300" />
                        {shift.break_duration_minutes} phút
                      </div>
                    )}
                  </td>
                  <td className="px-8 py-6 text-right">
                    <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button 
                        onClick={() => openEditModal(shift)}
                        className="p-3 bg-slate-50 text-slate-400 hover:text-blue-600 hover:bg-blue-50 rounded-2xl transition-all border border-transparent hover:border-blue-100"
                      >
                        <Save className="w-5 h-5" />
                      </button>
                      <button className="p-3 bg-slate-50 text-slate-400 hover:text-rose-600 hover:bg-rose-50 rounded-2xl transition-all border border-transparent hover:border-rose-100">
                        <Trash2 className="w-5 h-5" />
                      </button>
                    </div>
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
                  <div className="flex justify-between items-center ml-1">
                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">Giờ vào</label>
                    <span className="text-[9px] font-black text-blue-500 uppercase">{formatTime(newShiftStart)}</span>
                  </div>
                  <input 
                    type="time" 
                    step="60"
                    lang={workspace?.time_format === "12h" ? "en-US" : "vi-VN"}
                    value={newShiftStart?.substring(0, 5)}
                    onChange={(e) => setNewShiftStart(e.target.value)}
                    className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                  />
                </div>
                <div className="space-y-3">
                  <div className="flex justify-between items-center ml-1">
                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">Giờ ra</label>
                    <span className="text-[9px] font-black text-blue-500 uppercase">{formatTime(newShiftEnd)}</span>
                  </div>
                  <input 
                    type="time" 
                    step="60"
                    lang={workspace?.time_format === "12h" ? "en-US" : "vi-VN"}
                    value={newShiftEnd?.substring(0, 5)}
                    onChange={(e) => setNewShiftEnd(e.target.value)}
                    className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                  />
                </div>
              </div>

              <div className="bg-slate-50 p-5 rounded-3xl border border-slate-100 space-y-5">
                <div className="flex items-center justify-between">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest flex items-center gap-2">
                    <Clock size={12} /> Cấu hình nghỉ giải lao
                  </label>
                  <div className="flex items-center gap-2 bg-slate-200/50 p-1 rounded-xl">
                    <button 
                      onClick={() => setIsFixedBreak(false)}
                      className={`px-3 py-1 rounded-lg text-[10px] font-black uppercase tracking-tight transition-all ${!isFixedBreak ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-400 hover:text-slate-600'}`}
                    >
                      Linh hoạt
                    </button>
                    <button 
                      onClick={() => {
                        setIsFixedBreak(true);
                        if (!newBreakStart) setNewBreakStart("12:00");
                        if (!newBreakEnd) setNewBreakEnd("13:00");
                      }}
                      className={`px-3 py-1 rounded-lg text-[10px] font-black uppercase tracking-tight transition-all ${isFixedBreak ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-400 hover:text-slate-600'}`}
                    >
                      Cố định
                    </button>
                  </div>
                </div>

                {isFixedBreak ? (
                  <div className="space-y-4 animate-in fade-in slide-in-from-top-2">
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <div className="flex justify-between items-center ml-1">
                          <label className="text-[9px] font-black text-slate-400 uppercase tracking-widest">Bắt đầu nghỉ</label>
                          <span className="text-[8px] font-black text-indigo-500 uppercase">{formatTime(newBreakStart)}</span>
                        </div>
                        <input 
                          type="time" 
                          step="60"
                          lang={workspace?.time_format === "12h" ? "en-US" : "vi-VN"}
                          value={newBreakStart?.substring(0, 5)}
                          onChange={(e) => setNewBreakStart(e.target.value)}
                          className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 font-bold text-sm text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                        />
                      </div>
                      <div className="space-y-2">
                        <div className="flex justify-between items-center ml-1">
                          <label className="text-[9px] font-black text-slate-400 uppercase tracking-widest">Kết thúc nghỉ</label>
                          <span className="text-[8px] font-black text-indigo-500 uppercase">{formatTime(newBreakEnd)}</span>
                        </div>
                        <input 
                          type="time" 
                          step="60"
                          lang={workspace?.time_format === "12h" ? "en-US" : "vi-VN"}
                          value={newBreakEnd?.substring(0, 5)}
                          onChange={(e) => setNewBreakEnd(e.target.value)}
                          className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 font-bold text-sm text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                        />
                      </div>
                    </div>
                    
                    <label className="flex items-center gap-3 p-3 bg-white rounded-xl border border-slate-100 cursor-pointer group hover:border-blue-200 transition-all">
                      <input 
                        type="checkbox" 
                        checked={isAutoDeduct}
                        onChange={(e) => setIsAutoDeduct(e.target.checked)}
                        className="w-4 h-4 rounded-md border-slate-300 text-blue-600 focus:ring-blue-500"
                      />
                      <span className="text-xs font-bold text-slate-600 group-hover:text-blue-600 transition-colors">Tự động trừ giờ nghỉ (Auto-deduct)</span>
                    </label>
                  </div>
                ) : (
                  <div className="space-y-4 animate-in fade-in slide-in-from-top-2">
                     <div className="flex justify-between items-center px-1">
                        <span className="text-xs font-bold text-slate-700">Thời lượng nghỉ</span>
                        <span className="bg-blue-100 text-blue-700 px-2 py-0.5 rounded-lg text-[10px] font-black">{newShiftBreak} phút</span>
                      </div>
                      <input 
                        type="range" 
                        min="0" 
                        max="120" 
                        step="15"
                        value={newShiftBreak}
                        onChange={(e) => setNewShiftBreak(parseInt(e.target.value))}
                        className="w-full h-2 bg-slate-200 rounded-full appearance-none cursor-pointer accent-blue-600"
                      />
                  </div>
                )}
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

      {/* Modal: Edit Shift */}
      {editingShift && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/40 backdrop-blur-sm animate-in fade-in duration-200">
          <div className="bg-white rounded-[44px] shadow-2xl w-full max-w-lg overflow-hidden animate-in zoom-in-95 duration-200 border border-slate-100">
            <div className="p-10 border-b border-slate-50 flex justify-between items-center bg-slate-50/30">
              <div>
                <h3 className="text-2xl font-black text-slate-900 uppercase tracking-tight leading-tight">Chỉnh sửa: {editingShift.name}</h3>
                <p className="text-[10px] text-slate-400 font-black mt-1 uppercase tracking-widest">CẬP NHẬT CẤU HÌNH HOẶC THAY ĐỔI GIỜ LÀM</p>
              </div>
              <button onClick={() => setEditingShift(null)} className="p-3 hover:bg-white border border-transparent hover:border-slate-100 rounded-2xl transition-all">
                <X className="w-6 h-6 text-slate-300" />
              </button>
            </div>
            
            <div className="p-10 space-y-8 max-h-[70vh] overflow-y-auto">
              <div className="space-y-3">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Tên loại ca</label>
                <input 
                  type="text" 
                  value={newShiftName}
                  onChange={(e) => setNewShiftName(e.target.value)}
                  className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                />
              </div>

              <div className="grid grid-cols-2 gap-5">
                <div className="space-y-3">
                  <div className="flex justify-between items-center ml-1">
                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">Giờ vào</label>
                    <span className="text-[9px] font-black text-blue-500 uppercase">{formatTime(newShiftStart)}</span>
                  </div>
                  <input 
                    type="time" 
                    step="60"
                    lang={workspace?.time_format === "12h" ? "en-US" : "vi-VN"}
                    value={newShiftStart}
                    onChange={(e) => setNewShiftStart(e.target.value)}
                    className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                  />
                </div>
                <div className="space-y-3">
                  <div className="flex justify-between items-center ml-1">
                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">Giờ ra</label>
                    <span className="text-[9px] font-black text-blue-500 uppercase">{formatTime(newShiftEnd)}</span>
                  </div>
                  <input 
                    type="time" 
                    step="60"
                    lang={workspace?.time_format === "12h" ? "en-US" : "vi-VN"}
                    value={newShiftEnd}
                    onChange={(e) => setNewShiftEnd(e.target.value)}
                    className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                  />
                </div>
              </div>

              <div className="bg-slate-50 p-5 rounded-3xl border border-slate-100 space-y-5">
                <div className="flex items-center justify-between">
                  <label className="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-400">Nghỉ giải lao</label>
                  <div className="flex items-center gap-2 rounded-xl bg-slate-200/50 p-1">
                    <button 
                      onClick={() => setIsFixedBreak(false)}
                      className={`px-3 py-1 rounded-lg text-[10px] font-black uppercase transition-all ${!isFixedBreak ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-400'}`}
                    > Linh hoạt </button>
                    <button 
                      onClick={() => setIsFixedBreak(true)}
                      className={`px-3 py-1 rounded-lg text-[10px] font-black uppercase transition-all ${isFixedBreak ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-400'}`}
                    > Cố định </button>
                  </div>
                </div>
                {isFixedBreak ? (
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-1">
                      <div className="flex justify-between items-center px-1">
                        <span className="text-[8px] font-black text-slate-400 uppercase">Bắt đầu</span>
                        <span className="text-[8px] font-black text-blue-500 uppercase">{formatTime(newBreakStart)}</span>
                      </div>
                      <input type="time" step="60" lang={workspace?.time_format === "12h" ? "en-US" : "vi-VN"} value={newBreakStart} onChange={(e) => setNewBreakStart(e.target.value)} className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 font-bold text-sm" />
                    </div>
                    <div className="space-y-1">
                      <div className="flex justify-between items-center px-1">
                        <span className="text-[8px] font-black text-slate-400 uppercase">Kết thúc</span>
                        <span className="text-[8px] font-black text-blue-500 uppercase">{formatTime(newBreakEnd)}</span>
                      </div>
                      <input type="time" step="60" lang={workspace?.time_format === "12h" ? "en-US" : "vi-VN"} value={newBreakEnd} onChange={(e) => setNewBreakEnd(e.target.value)} className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 font-bold text-sm" />
                    </div>
                  </div>
                ) : (
                  <input type="range" min="0" max="120" step="15" value={newShiftBreak} onChange={(e) => setNewShiftBreak(parseInt(e.target.value))} className="w-full h-2 bg-slate-200 rounded-full appearance-none cursor-pointer accent-blue-600" />
                )}
              </div>

              {/* Versioning Logic */}
              <div className="bg-blue-50 p-6 rounded-[32px] border border-blue-100 space-y-4">
                <label className="group flex cursor-pointer items-start gap-3">
                  <input 
                    type="checkbox" 
                    checked={isEditCorrection}
                    onChange={(e) => setIsEditCorrection(e.target.checked)}
                    className="mt-1 h-5 w-5 rounded-lg border-blue-200 text-blue-600 focus:ring-blue-500"
                  />
                  <div className="flex-1">
                    <span className="block text-xs font-black uppercase tracking-tight text-blue-900">Sửa lỗi nhập liệu (Correction)</span>
                    <span className="mt-0.5 block text-[10px] font-bold leading-tight text-blue-500">
                      Thông tin sẽ được ghi đè trực tiếp lên bản ghi hiện tại. <br/> 
                      <span className="text-[9px] italic font-black text-rose-500 underline">Cảnh báo: Có thể làm thay đổi dữ liệu chấm công quá khứ.</span>
                    </span>
                  </div>
                </label>

                {!isEditCorrection && (
                  <div className="animate-in slide-in-from-top-2 space-y-2 border-t border-blue-100 pt-2">
                    <label className="ml-1 text-[10px] font-black uppercase tracking-widest text-blue-900">Ngày áp dụng thay đổi</label>
                    <input 
                      type="date" 
                      value={editEffectiveDate}
                      onChange={(e) => setEditEffectiveDate(e.target.value)}
                      className="w-full rounded-2xl border border-blue-200 bg-white px-5 py-3 font-bold text-blue-700 outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <p className="px-1 text-[9px] font-bold italic text-blue-400">
                      Hệ thống sẽ tạo phiên bản mới và chuyển toàn bộ nhân viên sang ca này từ ngày đã chọn.
                    </p>
                  </div>
                )}
              </div>

              <button
                onClick={handleUpdateShift}
                disabled={updateShiftMutation.isPending}
                className="active:scale-95 shadow-xl shadow-slate-200 w-full rounded-[28px] bg-slate-900 py-5 text-xs font-black uppercase tracking-widest text-white transition-all hover:bg-black flex items-center justify-center gap-3"
              >
                <Save className="h-5 w-5" />
                Cập nhật cấu hình
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Modal: Shift Picker */}
      {isShiftPickerOpen && (
        <div className="fixed inset-0 z-[60] flex items-center justify-center p-4 bg-slate-900/40 backdrop-blur-sm animate-in fade-in duration-200">
          <div className="bg-white rounded-[44px] shadow-2xl w-full max-w-2xl overflow-hidden animate-in zoom-in-95 duration-200 border border-slate-100">
            <div className="p-8 border-b border-slate-50 flex justify-between items-center bg-slate-50/30">
              <div>
                <h3 className="text-xl font-black text-slate-900 uppercase tracking-tight">Chọn ca làm việc</h3>
                <p className="text-[10px] text-slate-400 font-black mt-1 uppercase tracking-widest">
                  Gán cho {targetDates.length === 1 ? `ngày ${formatDate(targetDates[0])}` : `${targetDates.length} ngày đã chọn`}
                </p>
              </div>
              <button onClick={() => setIsShiftPickerOpen(false)} className="p-3 hover:bg-white border border-transparent hover:border-slate-100 rounded-2xl transition-all">
                <X className="w-6 h-6 text-slate-300" />
              </button>
            </div>
            
            <div className="p-8">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 max-h-[50vh] overflow-y-auto pr-2 custom-scrollbar">
                {/* Day Off Option */}
                <button
                  onClick={() => {
                    const items = targetDates.map(date => ({ date, shift_id: null }));
                    bulkRosterMutation.mutate(items);
                    setIsShiftPickerOpen(false);
                  }}
                  className="p-5 rounded-[28px] border-2 border-slate-100 hover:border-rose-200 hover:bg-rose-50/30 text-left transition-all group flex flex-col items-center justify-center gap-2 border-dashed"
                >
                  <div className="w-10 h-10 bg-slate-100 rounded-xl flex items-center justify-center text-slate-400 group-hover:bg-rose-100 group-hover:text-rose-500 transition-colors">
                    <X size={20} />
                  </div>
                  <div className="font-black text-xs uppercase tracking-widest text-slate-400 group-hover:text-rose-600">Đặt nghỉ (OFF)</div>
                </button>

                {/* Shift Options */}
                {shifts?.map(shift => (
                  <button
                    key={shift.id}
                    onClick={() => {
                      const items = targetDates.map(date => ({ date, shift_id: shift.id }));
                      bulkRosterMutation.mutate(items);
                      setIsShiftPickerOpen(false);
                    }}
                    className="p-5 rounded-[28px] border-2 border-slate-100 hover:border-blue-600 hover:bg-blue-50/50 text-left transition-all relative overflow-hidden group"
                  >
                    <div className="font-black text-sm uppercase tracking-tight text-slate-700 group-hover:text-blue-700">{shift.name}</div>
                    <div className="flex flex-wrap gap-x-3 gap-y-1 mt-1.5">
                                          <div className="text-[10px] text-slate-400 font-bold flex items-center gap-1 group-hover:text-blue-500">
                                            <Clock size={10} /> {formatTime(shift.start_time)} - {formatTime(shift.end_time)}
                                          </div>                      <div className="text-[10px] text-slate-400 font-bold flex items-center gap-1 group-hover:text-blue-500">
                        <Info size={10} /> 
                        {shift.break_start ? (
                          <span>{shift.break_start.substring(0, 5)}-{shift.break_end?.substring(0, 5)}</span>
                        ) : (
                          <span>Nghỉ {shift.break_duration_minutes}p</span>
                        )}
                      </div>
                    </div>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}