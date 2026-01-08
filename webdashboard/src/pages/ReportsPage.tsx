import React, { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { FileBarChart, Download, Calendar, Search } from "lucide-react";
import { 
  fetchAttendanceReport, 
  getCurrentWorkspaceId,
  AttendanceSummary
} from "../lib/api";

export default function ReportsPage() {
  const wsId = getCurrentWorkspaceId();
  
  // Default to current month
  const now = new Date();
  const firstDay = new Date(now.getFullYear(), now.getMonth(), 1).toISOString().split('T')[0];
  const lastDay = new Date(now.getFullYear(), now.getMonth() + 1, 0).toISOString().split('T')[0];

  const [dateRange, setDateRange] = useState({ start: firstDay, end: lastDay });
  const [searchTerm, setSearchTerm] = useState("");

  const { data: report = [], isLoading } = useQuery({
    queryKey: ["attendance-report", wsId, dateRange],
    queryFn: () => fetchAttendanceReport(wsId!, dateRange.start, dateRange.end),
    enabled: !!wsId,
  });

  const filteredReport = report.filter(row => 
    row.employee_name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  if (!wsId) return <div className="p-8 text-center">Vui lòng chọn Workspace</div>;

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Báo cáo chấm công</h1>
          <p className="text-slate-500">Tổng hợp giờ làm và nghỉ phép theo khoảng thời gian.</p>
        </div>
        <button 
          className="flex items-center gap-2 border border-slate-200 bg-white px-4 py-2 rounded-xl hover:bg-slate-50 transition-colors shadow-sm font-semibold"
          onClick={() => alert('Chức năng xuất báo cáo sẽ sớm được ra mắt')}
        >
          <Download className="w-5 h-5 text-slate-600" />
          <span>Xuất CSV</span>
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
            ) : filteredReport.length === 0 ? (
              <tr><td colSpan={4} className="px-6 py-8 text-center text-slate-500 italic">Không có dữ liệu trong khoảng thời gian này.</td></tr>
            ) : filteredReport.map((row) => (
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
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-blue-50 rounded-3xl p-6 border border-blue-100">
          <h3 className="text-blue-800 font-bold mb-2 flex items-center gap-2">
            <FileBarChart className="w-5 h-5" /> Thống kê nhanh
          </h3>
          <p className="text-blue-600 text-sm">
            Tổng cộng có <strong>{filteredReport.length}</strong> nhân viên trong danh sách. 
            Tổng giờ làm việc: <strong>{filteredReport.reduce((acc, r) => acc + r.total_hours, 0).toFixed(1)}h</strong>.
          </p>
        </div>
      </div>
    </div>
  );
}
