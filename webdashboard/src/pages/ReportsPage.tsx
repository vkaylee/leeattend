import React, { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { FileBarChart, Download, Calendar, Search, List, TableProperties, Clock, User as UserIcon, CalendarDays } from "lucide-react";
import { 
  fetchAttendanceReport, 
  getCurrentWorkspaceId,
  AttendanceSummary,
  DailyAttendanceReport
} from "../lib/api";

export default function ReportsPage() {
  const wsId = getCurrentWorkspaceId();
  
  // Default to current month
  const now = new Date();
  const firstDay = new Date(now.getFullYear(), now.getMonth(), 1).toISOString().split('T')[0];
  const lastDay = new Date(now.getFullYear(), now.getMonth() + 1, 0).toISOString().split('T')[0];

  const [dateRange, setDateRange] = useState({ start: firstDay, end: lastDay });
  const [searchTerm, setSearchTerm] = useState("");
  const [reportType, setReportType] = useState<"summary" | "daily">("summary");

  const { data: reportData = [], isLoading } = useQuery({
    queryKey: ["attendance-report", wsId, dateRange, reportType],
    queryFn: () => fetchAttendanceReport(wsId!, dateRange.start, dateRange.end, reportType),
    enabled: !!wsId,
  });

  const filteredData = Array.isArray(reportData) ? reportData.filter((row: any) => 
    row.employee_name.toLowerCase().includes(searchTerm.toLowerCase())
  ) : [];

  if (!wsId) return <div className="p-8 text-center">Vui lòng chọn Workspace</div>;

  const formatTime = (isoString: string | null) => {
    if (!isoString) return "-";
    return new Date(isoString).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  const getStatusBadge = (status: string) => {
    const styles: Record<string, string> = {
      present: "bg-emerald-100 text-emerald-700",
      late: "bg-amber-100 text-amber-700",
      absent: "bg-rose-100 text-rose-700",
      off: "bg-slate-100 text-slate-600",
    };
    const labels: Record<string, string> = {
      present: "Đúng giờ",
      late: "Đi muộn",
      absent: "Vắng mặt",
      off: "Ngày nghỉ",
    };
    return (
      <span className={`px-2 py-1 rounded-full text-[10px] font-black uppercase tracking-wider ${styles[status] || styles.off}`}>
        {labels[status] || status}
      </span>
    );
  };

  return (
    <div className="space-y-6 pb-20">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Báo cáo chấm công</h1>
          <p className="text-slate-500">Xem và xuất báo cáo dữ liệu chấm công của nhân viên.</p>
        </div>
        <button 
          className="flex items-center gap-2 border border-slate-200 bg-white px-4 py-2 rounded-xl hover:bg-slate-50 transition-colors shadow-sm font-semibold"
          onClick={() => alert('Chức năng xuất báo cáo sẽ sớm được ra mắt')}
        >
          <Download className="w-5 h-5 text-slate-600" />
          <span>Xuất Excel</span>
        </button>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 bg-slate-100 p-1 rounded-2xl w-fit">
        <button
          onClick={() => setReportType("summary")}
          className={`flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-bold transition-all ${
            reportType === "summary" ? "bg-white shadow-sm text-slate-900" : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <List className="w-4 h-4" />
          Tổng hợp tháng
        </button>
        <button
          onClick={() => setReportType("daily")}
          className={`flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-bold transition-all ${
            reportType === "daily" ? "bg-white shadow-sm text-slate-900" : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <TableProperties className="w-4 h-4" />
          Chi tiết hàng ngày
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 bg-white border border-slate-200 rounded-3xl p-6 shadow-sm">
        <div className="space-y-1">
          <label className="text-sm font-semibold text-slate-700 flex items-center gap-2">
            <Calendar className="w-4 h-4 text-blue-600" /> Từ ngày
          </label>
          <input 
            type="date" 
            value={dateRange.start}
            onChange={(e) => setDateRange(prev => ({ ...prev, start: e.target.value }))}
            className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none focus:ring-2 focus:ring-blue-500/20"
          />
        </div>
        <div className="space-y-1">
          <label className="text-sm font-semibold text-slate-700 flex items-center gap-2">
            <Calendar className="w-4 h-4 text-blue-600" /> Đến ngày
          </label>
          <input 
            type="date" 
            value={dateRange.end}
            onChange={(e) => setDateRange(prev => ({ ...prev, end: e.target.value }))}
            className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none focus:ring-2 focus:ring-blue-500/20"
          />
        </div>
        <div className="space-y-1">
          <label className="text-sm font-semibold text-slate-700 flex items-center gap-2">
            <Search className="w-4 h-4 text-blue-600" /> Tìm nhân viên
          </label>
          <input 
            type="text" 
            placeholder="Tên nhân viên..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none focus:ring-2 focus:ring-blue-500/20"
          />
        </div>
      </div>

      <div className="bg-white border border-slate-200 rounded-3xl overflow-hidden shadow-sm">
        {reportType === "summary" ? (
          <table className="w-full text-left">
            <thead className="bg-slate-50 border-b border-slate-200">
              <tr>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">Nhân viên</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">Tổng giờ làm</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">Số ngày nghỉ</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">Ghi chú</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {isLoading ? (
                <tr><td colSpan={4} className="px-6 py-8 text-center text-slate-500">Đang tạo báo cáo...</td></tr>
              ) : filteredData.length === 0 ? (
                <tr><td colSpan={4} className="px-6 py-8 text-center text-slate-500 italic">Không có dữ liệu trong khoảng thời gian này.</td></tr>
              ) : filteredData.map((row: AttendanceSummary) => (
                <tr key={row.employee_id} className="hover:bg-slate-50/50 transition-colors">
                  <td className="px-6 py-4 font-medium text-slate-900">{row.employee_name}</td>
                  <td className="px-6 py-4 text-right font-mono text-blue-600 font-bold">
                    {row.total_hours.toFixed(1)}h
                  </td>
                  <td className="px-6 py-4 text-right font-mono text-orange-600 font-bold">
                    {row.total_leave_days} ngày
                  </td>
                  <td className="px-6 py-4 text-right text-sm text-slate-400 italic">
                    -
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left min-w-[800px]">
              <thead className="bg-slate-50 border-b border-slate-200">
                <tr>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 whitespace-nowrap">Ngày</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600">Nhân viên</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 whitespace-nowrap">Ca làm việc</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 whitespace-nowrap text-center">Punch In</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 whitespace-nowrap text-center">Punch Out</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">Giờ làm</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-center">Trạng thái</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {isLoading ? (
                  <tr><td colSpan={7} className="px-6 py-8 text-center text-slate-500">Đang tải chi tiết...</td></tr>
                ) : filteredData.length === 0 ? (
                  <tr><td colSpan={7} className="px-6 py-8 text-center text-slate-500 italic">Không có dữ liệu trong khoảng thời gian này.</td></tr>
                ) : filteredData.map((row: DailyAttendanceReport, idx: number) => (
                  <tr key={`${row.employee_id}-${row.date}-${idx}`} className="hover:bg-slate-50/50 transition-colors">
                    <td className="px-6 py-4 text-sm font-medium text-slate-600 whitespace-nowrap font-mono uppercase">
                      {new Date(row.date).toLocaleDateString('vi-VN', { weekday: 'short', day: '2-digit', month: '2-digit' })}
                    </td>
                    <td className="px-6 py-4 font-bold text-slate-900">{row.employee_name}</td>
                    <td className="px-6 py-4 text-sm text-slate-500 whitespace-nowrap">
                      {row.scheduled_start ? (
                        <div className="flex flex-col">
                          <span className="font-semibold text-slate-700">{row.scheduled_start.substring(0, 5)} - {row.scheduled_end?.substring(0, 5)}</span>
                        </div>
                      ) : <span className="text-slate-400">Không có ca</span>}
                    </td>
                    <td className="px-6 py-4 text-center">
                      <div className="flex flex-col items-center">
                        <span className={`font-mono font-bold ${row.actual_punch_in ? "text-slate-900" : "text-slate-300"}`}>
                          {formatTime(row.actual_punch_in)}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-center">
                      <div className="flex flex-col items-center">
                        <span className={`font-mono font-bold ${row.actual_punch_out ? "text-slate-900" : "text-slate-300"}`}>
                          {formatTime(row.actual_punch_out)}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-right font-mono text-blue-600 font-black">
                      {(row.total_minutes / 60).toFixed(1)}h
                    </td>
                    <td className="px-6 py-4 text-center">
                      {getStatusBadge(row.status)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-slate-900 rounded-3xl p-6 border border-slate-800 shadow-xl shadow-slate-200">
          <h3 className="text-white font-bold mb-2 flex items-center gap-2">
            <FileBarChart className="w-5 h-5 text-blue-400" /> Thống kê nhanh
          </h3>
          <div className="grid grid-cols-2 gap-4 mt-4">
            <div className="bg-slate-800/50 p-4 rounded-2xl">
              <p className="text-slate-400 text-xs font-black uppercase tracking-widest">Nhân viên</p>
              <p className="text-2xl font-bold text-white mt-1">{new Set(filteredData.map((r: any) => r.employee_id)).size}</p>
            </div>
            <div className="bg-slate-800/50 p-4 rounded-2xl">
              <p className="text-slate-400 text-xs font-black uppercase tracking-widest">Tổng giờ</p>
              <p className="text-2xl font-bold text-blue-400 mt-1">
                {reportType === "summary" 
                  ? filteredData.reduce((acc: number, r: any) => acc + r.total_hours, 0).toFixed(1)
                  : (filteredData.reduce((acc: number, r: any) => acc + r.total_minutes, 0) / 60).toFixed(1)}h
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
