import { useState } from "react";
import { AlertTriangle, RefreshCcw, FileText, Calendar, Clock, Banknote } from "lucide-react";
import { clearWorkspaceData, getCurrentWorkspaceId } from "../lib/api";
import { useToast } from "./Toast";

export default function DebugWorkspaceTools() {
  const wsId = getCurrentWorkspaceId();
  const toast = useToast();
  const [loading, setLoading] = useState<string | null>(null);

  const handleClear = async (group: string, label: string) => {
    if (!wsId) return;
    if (!confirm(`Bạn có chắc chắn muốn xóa TOÀN BỘ dữ liệu [${label}] của Workspace này không? Thao tác này không thể hoàn tác.`)) {
      return;
    }

    setLoading(group);
    try {
      await clearWorkspaceData(wsId, group);
      toast.success(`Đã xóa dữ liệu [${label}] thành công.`);
    } catch (e: any) {
      toast.error(`Lỗi khi xóa dữ liệu: ${e.message}`);
    } finally {
      setLoading(null);
    }
  };

  const tools = [
    { id: "attendance", label: "Chấm công", icon: Clock, color: "text-amber-500", bg: "bg-amber-50" },
    { id: "schedules", label: "Lịch trình", icon: Calendar, color: "text-blue-500", bg: "bg-primary-50" },
    { id: "leave", label: "Nghỉ phép", icon: FileText, color: "text-emerald-500", bg: "bg-emerald-50" },
    { id: "payroll", label: "Lương & Hợp đồng", icon: Banknote, color: "text-purple-500", bg: "bg-purple-50" },
    { id: "full_reset", label: "FULL RESET (Cẩn thận!)", icon: AlertTriangle, color: "text-rose-500", bg: "bg-rose-50" },
  ];

  return (
    <div className="bg-white p-8 rounded-3xl border border-rose-100 shadow-sm">
      <div className="mb-6">
        <h2 className="text-xl font-bold flex items-center gap-2 text-rose-600">
          <RefreshCcw className="w-5 h-5" />
          Debug Tools (Chỉ dành cho Dev)
        </h2>
        <p className="text-slate-500 text-sm mt-1">
          Các công cụ dọn dẹp nhanh dữ liệu để hỗ trợ testing. 
          <span className="font-bold text-rose-500 ml-1">Cảnh báo: Dữ liệu bị xóa sẽ không thể phục hồi!</span>
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {tools.map((tool) => (
          <button
            key={tool.id}
            onClick={() => handleClear(tool.id, tool.label)}
            disabled={!!loading}
            className={`flex flex-col items-start p-4 rounded-2xl border border-slate-100 hover:border-rose-200 transition-all text-left group disabled:opacity-50`}
          >
            <div className={`p-3 rounded-xl ${tool.bg} ${tool.color} mb-3 group-hover:scale-110 transition-transform`}>
              <tool.icon className="w-5 h-5" />
            </div>
            <span className="font-bold text-slate-700">{tool.label}</span>
            <span className="text-[10px] text-slate-400 font-medium uppercase tracking-wider mt-1">
              {loading === tool.id ? "Đang xóa..." : "Click để dọn dẹp"}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
