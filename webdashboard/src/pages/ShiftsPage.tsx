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
  Info,
  X
} from "lucide-react";
import { 
  fetchEmployees, 
  fetchShifts, 
  assignShift, 
  createShift,
  getCurrentWorkspaceId,
  Employee,
  Shift
} from "../lib/api";
import { useToast } from "../components/Toast";

export default function ShiftsPage() {
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  const [activeTab, setActiveTab] = useState<"roster" | "directory">("roster");
  
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
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

  const assignMutation = useMutation({
    mutationFn: (data: any) => assignShift(wsId, data),
    onSuccess: () => {
      toast.success("Đã gán ca làm việc cho nhân viên thành công.");
      // Reset form
      setSelectedShiftId("");
      setEndDate("");
      setIsForever(true);
      queryClient.invalidateQueries({ queryKey: ["employee_shifts"] });
    },
    onError: (error: any) => {
      toast.error(error.message, "Lỗi gán ca");
    }
  });

  const createShiftMutation = useMutation({
    mutationFn: (data: any) => createShift(wsId, data),
    onSuccess: () => {
      toast.success(`Loại ca "${newShiftName}" đã được khởi tạo.`);
      setIsCreateModalOpen(false);
      // Reset form
      setNewShiftName("");
      setNewShiftStart("08:00");
      setNewShiftEnd("17:00");
      setNewShiftBreak(60);
      queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
    },
    onError: (error: any) => {
      toast.error(error.message, "Lỗi tạo ca");
    }
  });

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
      <div className="flex justify-between items-end">
        <div>
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">Quản lý Ca làm việc</h1>
          <p className="text-slate-500 mt-1">Thiết lập danh mục ca và gán lịch cho nhân sự</p>
        </div>
        <div className="flex gap-3">
          <div className="bg-slate-100 p-1.5 rounded-2xl flex">
            <button 
              onClick={() => setActiveTab("roster")}
              className={`px-4 py-2 rounded-xl text-xs font-black transition-all ${activeTab === 'roster' ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
            >
              GÁN LỊCH TRỰC
            </button>
            <button 
              onClick={() => setActiveTab("directory")}
              className={`px-4 py-2 rounded-xl text-xs font-black transition-all ${activeTab === 'directory' ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
            >
              DANH MỤC CA
            </button>
          </div>
          <button 
            onClick={() => setIsCreateModalOpen(true)}
            className="bg-blue-600 hover:bg-blue-700 text-white font-black px-6 py-3 rounded-2xl flex items-center gap-2 shadow-lg shadow-blue-100 transition-all active:scale-95"
          >
            <Plus className="w-5 h-5" />
            THÊM LOẠI CA
          </button>
        </div>
      </div>

      {activeTab === "roster" ? (
        <div className="grid grid-cols-12 gap-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
          {/* Left: Employee List */}
          <div className="col-span-4 bg-white border border-slate-200 rounded-[32px] overflow-hidden shadow-sm flex flex-col">
            <div className="p-5 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
              <span className="font-bold text-slate-700">Danh sách nhân viên</span>
              <span className="bg-white px-2 py-1 rounded-lg text-[10px] font-black text-slate-400 border border-slate-200">
                {employees?.length || 0} NGƯỜI
              </span>
            </div>
            <div className="flex-1 overflow-y-auto max-h-[600px] divide-y divide-slate-50">
              {employees?.map(emp => (
                <button
                  key={emp.id}
                  onClick={() => setSelectedEmployee(emp)}
                  className={`w-full flex items-center gap-4 p-4 hover:bg-slate-50 transition-all text-left group ${selectedEmployee?.id === emp.id ? 'bg-blue-50/50' : ''}`}
                >
                  <div className={`w-12 h-12 rounded-2xl flex items-center justify-center font-bold text-lg transition-transform group-active:scale-95 ${selectedEmployee?.id === emp.id ? 'bg-blue-600 text-white shadow-lg shadow-blue-200' : 'bg-slate-100 text-slate-500'}`}>
                    {emp.last_name[0]}
                  </div>
                  <div className="flex-1">
                    <div className={`font-bold text-sm ${selectedEmployee?.id === emp.id ? 'text-blue-700' : 'text-slate-900'}`}>
                      {emp.first_name} {emp.last_name}
                    </div>
                    <div className="text-[11px] text-slate-400 font-medium">{emp.employee_id_number}</div>
                  </div>
                  <ChevronRight className={`w-4 h-4 transition-transform ${selectedEmployee?.id === emp.id ? 'text-blue-400 translate-x-1' : 'text-slate-200'}`} />
                </button>
              ))}
            </div>
          </div>

          {/* Right: Assignment Form */}
          <div className="col-span-8 space-y-6">
            {!selectedEmployee ? (
              <div className="bg-slate-50 border-2 border-dashed border-slate-200 rounded-[40px] h-[500px] flex flex-col items-center justify-center text-center p-12">
                <div className="bg-white p-6 rounded-[32px] shadow-sm mb-6">
                  <CalendarRange className="w-12 h-12 text-blue-500" />
                </div>
                <h3 className="text-xl font-bold text-slate-900">Chọn nhân sự để bắt đầu</h3>
                <p className="text-slate-500 mt-2 max-w-xs mx-auto text-sm leading-relaxed">
                  Chọn một nhân viên từ danh sách bên trái để thiết lập ca làm việc cố định hoặc tạm thời cho họ.
                </p>
              </div>
            ) : (
              <div className="bg-white border border-slate-200 rounded-[40px] p-8 shadow-sm">
                <div className="flex items-center gap-4 mb-8">
                  <div className="w-16 h-16 bg-blue-600 rounded-[24px] flex items-center justify-center text-white text-2xl font-black">
                    {selectedEmployee.last_name[0]}
                  </div>
                  <div>
                    <h2 className="text-2xl font-bold text-slate-900">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
                    <p className="text-slate-400 text-sm font-medium">Mã NV: {selectedEmployee.employee_id_number}</p>
                  </div>
                </div>

                <div className="space-y-8">
                  {/* Shift Selection */}
                  <div className="space-y-3">
                    <div className="flex justify-between items-center">
                      <label className="text-xs font-black text-slate-400 uppercase tracking-widest flex items-center gap-2">
                        <Clock className="w-3 h-3" /> Chọn Ca Làm Việc
                      </label>
                      <span className="text-[10px] font-bold text-slate-400">ĐÃ KHAI BÁO: {shifts?.length || 0}</span>
                    </div>
                    <div className="grid grid-cols-2 gap-3">
                      {shifts?.map(shift => (
                        <button
                          key={shift.id}
                          onClick={() => setSelectedShiftId(shift.id)}
                          className={`p-4 rounded-[24px] border-2 text-left transition-all relative overflow-hidden group ${selectedShiftId === shift.id ? 'border-blue-600 bg-blue-50/50' : 'border-slate-100 hover:border-slate-200'}`}
                        >
                          <div className={`font-bold text-sm ${selectedShiftId === shift.id ? 'text-blue-700' : 'text-slate-700'}`}>{shift.name}</div>
                          <div className="text-[10px] text-slate-400 font-mono mt-1">
                            {shift.start_time.substring(0, 5)} - {shift.end_time.substring(0, 5)}
                          </div>
                          {selectedShiftId === shift.id && (
                            <div className="absolute top-3 right-3 w-2 h-2 bg-blue-600 rounded-full animate-pulse" />
                          )}
                        </button>
                      ))}
                      {shifts?.length === 0 && (
                        <div className="col-span-2 p-6 border-2 border-dashed border-slate-100 rounded-[24px] text-center">
                          <p className="text-xs text-slate-400 font-medium">Chưa có loại ca nào. Hãy nhấn "Thêm loại ca" ở trên.</p>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Date Settings */}
                  <div className="grid grid-cols-2 gap-6">
                    <div className="space-y-3">
                      <label className="text-xs font-black text-slate-400 uppercase tracking-widest flex items-center gap-2">
                        <Calendar className="w-3 h-3" /> Ngày Bắt Đầu
                      </label>
                      <input 
                        type="date" 
                        value={startDate}
                        onChange={(e) => setStartDate(e.target.value)}
                        className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-4 py-3 font-bold text-slate-700 focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                      />
                    </div>

                    <div className="space-y-3">
                      <div className="flex justify-between items-center">
                        <label className="text-xs font-black text-slate-400 uppercase tracking-widest flex items-center gap-2">
                          <Calendar className="w-3 h-3" /> Ngày Kết Thúc
                        </label>
                        <button 
                          onClick={() => setIsForever(!isForever)}
                          className={`text-[10px] font-black px-2 py-0.5 rounded-lg border transition-all ${isForever ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-100 text-slate-400 border-slate-200'}`}
                        >
                          {isForever ? "MÃI MÃI" : "CÓ HẠN"}
                        </button>
                      </div>
                      <input 
                        type="date" 
                        disabled={isForever}
                        value={endDate}
                        onChange={(e) => setEndDate(e.target.value)}
                        className={`w-full border rounded-2xl px-4 py-3 font-bold transition-all outline-none ${isForever ? 'bg-slate-100 text-slate-300 border-slate-100' : 'bg-slate-50 text-slate-700 border-slate-200 focus:ring-2 focus:ring-blue-500'}`}
                      />
                    </div>
                  </div>

                  {/* Submit */}
                  <button
                    onClick={handleAssign}
                    disabled={assignMutation.isPending}
                    className="w-full bg-slate-900 text-white font-black py-5 rounded-[24px] flex items-center justify-center gap-3 hover:bg-black transition-all active:scale-[0.98] disabled:opacity-50 shadow-xl shadow-slate-200"
                  >
                    <Save className="w-5 h-5" />
                    XÁC NHẬN GÁN CA
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      ) : (
        <div className="bg-white border border-slate-200 rounded-[40px] overflow-hidden shadow-sm animate-in fade-in slide-in-from-bottom-4 duration-500">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50/50">
                <th className="p-6 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100">Tên loại ca</th>
                <th className="p-6 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100">Giờ làm việc</th>
                <th className="p-6 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100">Thời gian nghỉ</th>
                <th className="p-6 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100 text-right">Thao tác</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {shifts?.map(shift => (
                <tr key={shift.id} className="hover:bg-slate-50/50 transition-colors">
                  <td className="p-6">
                    <div className="font-bold text-slate-900">{shift.name}</div>
                    <div className="text-[10px] text-slate-400 font-mono mt-0.5">{shift.id}</div>
                  </td>
                  <td className="p-6">
                    <div className="flex items-center gap-2">
                      <div className="bg-blue-50 text-blue-600 px-3 py-1 rounded-lg font-mono font-bold text-sm">
                        {shift.start_time.substring(0, 5)}
                      </div>
                      <ChevronRight className="w-3 h-3 text-slate-300" />
                      <div className="bg-blue-50 text-blue-600 px-3 py-1 rounded-lg font-mono font-bold text-sm">
                        {shift.end_time.substring(0, 5)}
                      </div>
                    </div>
                  </td>
                  <td className="p-6">
                    <div className="flex items-center gap-2 font-bold text-sm text-slate-600">
                      <Clock className="w-4 h-4 text-slate-300" />
                      {shift.break_duration_minutes} phút
                    </div>
                  </td>
                  <td className="p-6 text-right">
                    <button className="p-2 hover:bg-slate-100 rounded-xl text-slate-400 hover:text-red-500 transition-all">
                      <Trash2 className="w-5 h-5" />
                    </button>
                  </td>
                </tr>
              ))}
              {shifts?.length === 0 && (
                <tr>
                  <td colSpan={4} className="p-20 text-center">
                    <div className="bg-slate-50 w-16 h-16 rounded-3xl flex items-center justify-center mx-auto mb-4">
                      <Clock className="w-8 h-8 text-slate-300" />
                    </div>
                    <p className="text-slate-400 font-bold">Chưa có loại ca nào được khai báo.</p>
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
          <div className="bg-white rounded-[40px] shadow-2xl w-full max-w-md overflow-hidden animate-in zoom-in-95 duration-200">
            <div className="p-8 border-b border-slate-100 flex justify-between items-center bg-slate-50/50">
              <div>
                <h3 className="text-2xl font-black text-slate-900 uppercase tracking-tight">Khai báo ca mới</h3>
                <p className="text-xs text-slate-400 font-bold mt-1">THIẾT LẬP THAM SỐ CA LÀM VIỆC</p>
              </div>
              <button onClick={() => setIsCreateModalOpen(false)} className="p-2 hover:bg-white rounded-xl transition-colors">
                <X className="w-6 h-6 text-slate-400" />
              </button>
            </div>
            
            <div className="p-8 space-y-6">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Tên ca làm việc</label>
                <input 
                  type="text" 
                  placeholder="Ví dụ: Ca hành chính, Ca sáng..."
                  value={newShiftName}
                  onChange={(e) => setNewShiftName(e.target.value)}
                  className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all placeholder:text-slate-300"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Giờ vào</label>
                  <input 
                    type="time" 
                    value={newShiftStart}
                    onChange={(e) => setNewShiftStart(e.target.value)}
                    className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Giờ ra</label>
                  <input 
                    type="time" 
                    value={newShiftEnd}
                    onChange={(e) => setNewShiftEnd(e.target.value)}
                    className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1 flex justify-between">
                  <span>Thời gian nghỉ (phút)</span>
                  <span className="text-blue-600">{newShiftBreak} PHÚT</span>
                </label>
                <input 
                  type="range" 
                  min="0" 
                  max="120" 
                  step="15"
                  value={newShiftBreak}
                  onChange={(e) => setNewShiftBreak(parseInt(e.target.value))}
                  className="w-full h-2 bg-slate-100 rounded-lg appearance-none cursor-pointer accent-blue-600"
                />
              </div>

              <button
                onClick={handleCreateShift}
                disabled={createShiftMutation.isPending}
                className="w-full bg-blue-600 text-white font-black py-5 rounded-2xl flex items-center justify-center gap-3 hover:bg-blue-700 transition-all active:scale-95 shadow-lg shadow-blue-100 disabled:opacity-50 mt-4"
              >
                <Save className="w-5 h-5" />
                LƯU LOẠI CA
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
