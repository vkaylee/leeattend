import { useState, useEffect } from "react";
import { AuthorizedAgent } from "@/lib/api";
import { isAgentOnline } from "@/lib/utils";
import {
  Plus,
  ShieldCheck,
  ShieldAlert,
  ShieldX,
  Key,
  UserCheck,
  Globe,
  RefreshCcw,
  Edit2,
  Trash2,
  Activity,
  Eye,
  EyeOff,
  RotateCcw,
} from "lucide-react";
import AgentEditModal from "@/components/AgentEditModal";
import { useAuthorizedAgents } from "@/hooks/use-agents";
import { useToast } from "../components/Toast";

export default function AgentsPage() {
  const toast = useToast();
  // Add a local timer to refresh the UI every 15s to update online status
  const [, setTick] = useState(0);
  useEffect(() => {
    const interval = setInterval(() => setTick((t) => t + 1), 15000);
    return () => clearInterval(interval);
  }, []);

  const {
    agents,
    isLoading,
    isFetching,
    refetch,
    createAgent,
    updateTrust,
    deleteAgent,
    updateAgent,
    includeInactive,
    setIncludeInactive,
  } = useAuthorizedAgents();

  const [showAddForm, setShowAddForm] = useState(false);
  const [newAgent, setNewAgent] = useState({ id: "", name: "", token: "" });

  // State for editing
  const [editingAgent, setEditingAgent] = useState<AuthorizedAgent | null>(
    null,
  );
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);

  async function handleRestore(agent: AuthorizedAgent) {
    try {
      await updateAgent({ id: agent.id, agent: { is_active: true } });
      toast.success(`Đã khôi phục Agent ${agent.name}`);
    } catch (e) {
      toast.error("Lỗi khi khôi phục Agent.");
    }
  }

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createAgent(newAgent);
      toast.success("Đã khai báo Agent mới thành công.");
      setShowAddForm(false);
      setNewAgent({ id: "", name: "", token: "" });
    } catch (e) {
      toast.error("Lỗi khi thêm Agent. ID có thể đã tồn tại.", "Lỗi hệ thống");
    }
  }

  async function handleToggleTrust(agent: AuthorizedAgent) {
    const action = agent.is_trusted ? "thu hồi quyền" : "phê duyệt";
    if (!confirm(`Bạn có chắc chắn muốn ${action} cho Agent ${agent.id}?`))
      return;

    try {
      await updateTrust({ id: agent.id, is_trusted: !agent.is_trusted });
      toast.success(`Đã ${action} thành công cho Agent.`);
    } catch (e) {
      toast.error("Lỗi khi cập nhật trạng thái.");
    }
  }

  async function handleDelete(agent: AuthorizedAgent) {
    if (
      !confirm(
        `Bạn có chắc chắn muốn xóa Agent ${agent.name} (${agent.id})? \nThao tác này sẽ ngắt kết nối và ẩn Agent khỏi hệ thống.`,
      )
    )
      return;

    try {
      await deleteAgent(agent.id);
      toast.success(`Đã xóa Agent ${agent.name}.`);
    } catch (e) {
      toast.error("Lỗi khi xóa Agent.");
    }
  }

  function handleEdit(agent: AuthorizedAgent) {
    setEditingAgent(agent);
    setIsEditModalOpen(true);
  }

  function handleSave(updated: AuthorizedAgent) {
    setEditingAgent(updated);
  }

  const loading = isLoading || isFetching;

  return (
    <div className="p-8">
      <div className="max-w-6xl mx-auto">
        <div className="mb-8 flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-slate-900 tracking-tight">
              Quản lý Agent
            </h1>
            <p className="text-slate-500 mt-1">
              Phê duyệt và kiểm soát các điểm kết nối Edge Agent
            </p>
          </div>
          <div className="flex gap-3">
            <button
              onClick={() => setIncludeInactive(!includeInactive)}
              className={`px-4 py-2.5 rounded-2xl border font-bold flex items-center gap-2 transition-all ${includeInactive
                  ? "bg-slate-100 border-slate-300 text-slate-900 shadow-inner"
                  : "bg-white border-slate-200 text-slate-600 hover:bg-slate-50"
              }`}
              title={
                includeInactive ? "Ẩn Agent đã đóng" : "Hiện Agent đã đóng"
              }
            >
              {includeInactive ? (
                <Eye className="w-5 h-5" />
              ) : (
                <EyeOff className="w-5 h-5" />
              )}
              {includeInactive ? "Đang hiện lưu trữ" : "Hiện lưu trữ"}
            </button>
            <button
              onClick={() => refetch()}
              className="bg-white p-2.5 rounded-2xl border border-slate-200 hover:bg-slate-50 transition-colors text-slate-600"
            >
              <RefreshCcw
                className={`w-5 h-5 ${loading ? "animate-spin" : ""}`}
              />
            </button>
            <button
              onClick={() => setShowAddForm(!showAddForm)}
              className="bg-slate-900 text-white px-6 py-2.5 rounded-2xl font-bold flex items-center gap-2 hover:bg-slate-800 transition-all shadow-lg shadow-slate-200"
            >
              <Plus className="w-5 h-5" />
              Khai báo Agent
            </button>
          </div>
        </div>

        {showAddForm && (
          <div className="mb-8 bg-white p-8 rounded-3xl border border-slate-100 shadow-xl animate-in fade-in slide-in-from-top-4 duration-300">
            <h2 className="text-xl font-bold mb-6 flex items-center gap-2 text-slate-800">
              <Plus className="w-5 h-5 text-blue-500" />
              Khai báo Agent mới
            </h2>
            <form
              onSubmit={handleAdd}
              className="grid grid-cols-1 md:grid-cols-3 gap-6"
            >
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  Agent ID (Duy nhất)
                </label>
                <input
                  required
                  placeholder="Ví dụ: BRANCH_HN_01"
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                  value={newAgent.id}
                  onChange={(e) =>
                    setNewAgent({ ...newAgent, id: e.target.value })
                  }
                />
              </div>
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  Tên Chi nhánh/Vị trí
                </label>
                <input
                  required
                  placeholder="Ví dụ: Văn phòng Hà Nội"
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                  value={newAgent.name}
                  onChange={(e) =>
                    setNewAgent({ ...newAgent, name: e.target.value })
                  }
                />
              </div>
              <div className="flex items-end gap-4">
                <div className="space-y-2 flex-1">
                  <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                    Secret Token
                  </label>
                  <div className="relative">
                    <input
                      required
                      placeholder="Mật mã kết nối"
                      className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all pr-10"
                      value={newAgent.token}
                      onChange={(e) =>
                        setNewAgent({ ...newAgent, token: e.target.value })
                      }
                    />
                    <Key className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
                  </div>
                </div>
                <button
                  type="submit"
                  className="bg-blue-600 text-white px-8 py-3 rounded-xl font-bold hover:bg-blue-700 transition-all shadow-lg shadow-blue-100"
                >
                  Lưu
                </button>
              </div>
            </form>
          </div>
        )}

        <div className="bg-white rounded-3xl border border-slate-100 overflow-hidden shadow-sm">
          <table className="w-full text-left">
            <thead>
              <tr className="bg-slate-50 border-b border-slate-100">
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">
                  Định danh
                </th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">
                  Token
                </th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">
                  IP cuối
                </th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">
                  Trạng thái
                </th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest text-right">
                  Thao tác
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {agents.map((agent) => (
                <tr
                  key={agent.id}
                  className={`hover:bg-slate-50/50 transition-colors group ${!agent.is_active ? "opacity-60 bg-slate-50/30" : ""}`}
                >
                  <td className="px-6 py-5">
                    <div className="flex items-center gap-3">
                      <div
                        className={`p-2 rounded-xl transition-all ${agent.is_active
                            ? "bg-slate-100 text-slate-500 group-hover:bg-white group-hover:shadow-sm"
                            : "bg-slate-200 text-slate-400"
                        }`}
                      >
                        <UserCheck className="w-5 h-5" />
                      </div>
                      <div>
                        <p
                          className={`font-bold ${agent.is_active ? "text-slate-900" : "text-slate-500"}`}
                        >
                          {agent.name}
                          {!agent.is_active && (
                            <span className="ml-2 text-[10px] bg-slate-200 text-slate-600 px-1.5 py-0.5 rounded uppercase tracking-tighter">
                              Đã đóng
                            </span>
                          )}
                        </p>
                        <p className="text-xs font-mono text-slate-400">
                          {agent.id}
                        </p>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-5">
                    <div className="flex items-center gap-2 text-slate-400 font-mono text-xs bg-slate-50 px-2 py-1 rounded-lg w-fit">
                      <Key className="w-3 h-3" />
                      {agent.token.replace(/./g, "•").slice(0, 12)}
                    </div>
                  </td>
                  <td className="px-6 py-5">
                    <div className="flex items-center gap-2 text-slate-500 text-sm">
                      <Globe className="w-4 h-4 text-slate-300" />
                      {agent.last_ip || "---"}
                    </div>
                  </td>
                  <td className="px-6 py-5">
                    <div className="flex flex-col gap-2">
                      {isAgentOnline(agent.last_seen_at) ? (
                        <span className="flex items-center gap-1.5 text-blue-600 font-bold text-[10px] bg-blue-50 px-3 py-1 rounded-full w-fit uppercase tracking-wider animate-pulse">
                          <Activity className="w-3 h-3" />
                          Online
                        </span>
                      ) : (
                        <span className="flex items-center gap-1.5 text-slate-400 font-bold text-[10px] bg-slate-100 px-3 py-1 rounded-full w-fit uppercase tracking-wider">
                          <Activity className="w-3 h-3" />
                          Offline
                        </span>
                      )}

                      {agent.is_trusted ? (
                        <span className="flex items-center gap-1.5 text-emerald-600 font-bold text-[10px] bg-emerald-50 px-3 py-1 rounded-full w-fit uppercase tracking-wider">
                          <ShieldCheck className="w-3 h-3" />
                          Trusted
                        </span>
                      ) : (
                        <span className="flex items-center gap-1.5 text-amber-600 font-bold text-[10px] bg-amber-50 px-3 py-1 rounded-full w-fit uppercase tracking-wider">
                          <ShieldAlert className="w-3 h-3" />
                          Pending
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="px-6 py-5 text-right">
                    <div className="flex items-center justify-end gap-2">
                      {agent.is_active ? (
                        <>
                          <button
                            onClick={() => handleEdit(agent)}
                            className="p-2 text-slate-400 hover:text-blue-600 hover:bg-blue-50 rounded-xl transition-all"
                            title="Chỉnh sửa"
                          >
                            <Edit2 className="w-4 h-4" />
                          </button>

                          <button
                            onClick={() => handleToggleTrust(agent)}
                            className={`px-3 py-1.5 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all border ${agent.is_trusted
                                ? "text-rose-600 hover:bg-rose-50 border-rose-100"
                                : "bg-blue-600 text-white hover:bg-blue-700 border-transparent shadow-md shadow-blue-100"
                            }`}
                          >
                            {agent.is_trusted ? "Revoke" : "Trust"}
                          </button>

                          <button
                            onClick={() => handleDelete(agent)}
                            className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 rounded-xl transition-all"
                            title="Xóa"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </>
                      ) : (
                        <button
                          onClick={() => handleRestore(agent)}
                          className="flex items-center gap-1 px-4 py-2 bg-emerald-600 text-white text-[10px] font-bold uppercase tracking-widest rounded-xl hover:bg-emerald-700 transition-all shadow-md shadow-emerald-100"
                        >
                          <RotateCcw className="w-3 h-3" />
                          Khôi phục
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {agents.length === 0 && !loading && (
            <div className="py-20 text-center text-slate-400">
              <ShieldX className="w-12 h-12 mx-auto mb-4 text-slate-200" />
              <p className="font-medium">Chưa có Agent nào được khai báo.</p>
            </div>
          )}
        </div>
      </div>

      <AgentEditModal
        agent={editingAgent}
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        onSave={handleSave}
      />
    </div>
  );
}