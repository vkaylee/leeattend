import { useState } from 'react';
import { Plus, Trash2, Server, Cpu, Globe, Hash, ShieldCheck, ShieldAlert, ArrowLeft } from 'lucide-react';
import { Link } from 'react-router-dom';
import { useDevices } from '@/hooks/use-devices';
import { useAgentsStatus } from '@/hooks/use-agents';

export default function DevicesPage() {
  const { devices, isLoading: devicesLoading, createDevice, deleteDevice } = useDevices();
  const { agents, isLoading: agentsLoading } = useAgentsStatus();

  const [showAddForm, setShowAddForm] = useState(false);
  const [newDevice, setNewDevice] = useState({
    name: '',
    agent_id: '',
    ip: '',
    port: 4370
  });

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createDevice(newDevice);
      setShowAddForm(false);
      setNewDevice({ name: '', agent_id: '', ip: '', port: 4370 });
    } catch (e) {
      alert('Lỗi khi thêm thiết bị');
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Bạn có chắc chắn muốn xóa thiết bị này?')) return;
    try {
      await deleteDevice(id);
    } catch (e) {
      alert('Lỗi khi xóa thiết bị');
    }
  }

  const loading = devicesLoading || agentsLoading;

  return (
    <div className="min-h-screen bg-slate-50 p-8">
      <div className="max-w-6xl mx-auto">
        <div className="mb-8 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Link to="/" className="bg-white p-2 rounded-xl border border-slate-200 hover:bg-slate-50 transition-colors">
              <ArrowLeft className="w-5 h-5 text-slate-600" />
            </Link>
            <h1 className="text-3xl font-bold text-slate-900 tracking-tight">Quản lý thiết bị</h1>
          </div>
          <button 
            onClick={() => setShowAddForm(!showAddForm)}
            className="bg-blue-600 text-white px-6 py-2.5 rounded-2xl font-bold flex items-center gap-2 hover:bg-blue-700 transition-colors shadow-lg shadow-blue-200"
          >
            <Plus className="w-5 h-5" />
            Thêm thiết bị
          </button>
        </div>

        {showAddForm && (
          <div className="mb-8 bg-white p-8 rounded-3xl border border-blue-100 shadow-xl shadow-blue-50 animate-in fade-in slide-in-from-top-4 duration-300">
            <h2 className="text-xl font-bold mb-6 flex items-center gap-2 text-slate-800">
              <Plus className="w-5 h-5 text-blue-500" />
              Thiết bị mới
            </h2>
            <form onSubmit={handleAdd} className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">Tên thiết bị</label>
                <input 
                  required
                  placeholder="Ví dụ: Máy cổng chính"
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                  value={newDevice.name}
                  onChange={e => setNewDevice({...newDevice, name: e.target.value})}
                />
              </div>
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">Chọn Agent</label>
                <select 
                  required
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all appearance-none"
                  value={newDevice.agent_id}
                  onChange={e => setNewDevice({...newDevice, agent_id: e.target.value})}
                >
                  <option value="">Chọn một Agent...</option>
                  {agents.map(a => (
                    <option key={a.agent_id} value={a.agent_id}>{a.agent_id}</option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">Địa chỉ IP</label>
                <input 
                  required
                  placeholder="192.168.1.100"
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                  value={newDevice.ip}
                  onChange={e => setNewDevice({...newDevice, ip: e.target.value})}
                />
              </div>
              <div className="flex items-end gap-4">
                <div className="space-y-2 flex-1">
                  <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">Cổng (Port)</label>
                  <input 
                    type="number"
                    className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                    value={newDevice.port}
                    onChange={e => setNewDevice({...newDevice, port: parseInt(e.target.value)})}
                  />
                </div>
                <button type="submit" className="bg-slate-900 text-white px-6 py-3 rounded-xl font-bold hover:bg-slate-800 transition-all shadow-lg shadow-slate-200">
                  Lưu
                </button>
              </div>
            </form>
          </div>
        )}

        {loading ? (
          <div className="py-20 text-center">
            <div className="animate-spin w-10 h-10 border-4 border-blue-600 border-t-transparent rounded-full mx-auto"></div>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-4">
            {devices.map(device => (
              <div key={device.id} className="bg-white p-6 rounded-3xl border border-slate-100 flex items-center gap-6 hover:shadow-md transition-all group">
                <div className="bg-slate-50 p-4 rounded-2xl group-hover:bg-blue-50 transition-colors">
                  <Cpu className="w-8 h-8 text-slate-400 group-hover:text-blue-500 transition-colors" />
                </div>
                
                <div className="flex-1 grid grid-cols-1 md:grid-cols-4 gap-4">
                  <div>
                    <p className="text-xs text-slate-400 font-bold uppercase tracking-widest mb-1">Thiết bị</p>
                    <h3 className="font-bold text-slate-800 text-lg">{device.name}</h3>
                    <p className="text-xs text-slate-400 font-mono mt-1">{device.id}</p>
                  </div>
                  
                  <div>
                    <p className="text-xs text-slate-400 font-bold uppercase tracking-widest mb-1">Phụ trách bởi</p>
                    <div className="flex items-center gap-2">
                      <Server className="w-4 h-4 text-slate-400" />
                      <span className="font-semibold text-slate-700">{device.agent_id}</span>
                    </div>
                  </div>
                  
                  <div>
                    <p className="text-xs text-slate-400 font-bold uppercase tracking-widest mb-1">Kết nối</p>
                    <div className="flex items-center gap-3">
                      <div className="flex items-center gap-1.5 text-slate-600 bg-slate-50 px-3 py-1 rounded-lg border border-slate-100">
                        <Globe className="w-3.5 h-3.5" />
                        <span className="text-sm font-bold">{device.ip}</span>
                      </div>
                      <div className="flex items-center gap-1 text-slate-400">
                        <Hash className="w-3.5 h-3.5" />
                        <span className="text-sm">{device.port}</span>
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex items-center justify-end gap-6">
                    <div className="flex items-center gap-2">
                      {device.is_enabled ? (
                        <div className="flex items-center gap-1.5 text-emerald-600 bg-emerald-50 px-3 py-1 rounded-full text-xs font-bold">
                          <ShieldCheck className="w-3.5 h-3.5" />
                          Hoạt động
                        </div>
                      ) : (
                        <div className="flex items-center gap-1.5 text-slate-400 bg-slate-50 px-3 py-1 rounded-full text-xs font-bold">
                          <ShieldAlert className="w-3.5 h-3.5" />
                          Đang tắt
                        </div>
                      )}
                    </div>
                    <button 
                      onClick={() => handleDelete(device.id)}
                      className="text-slate-300 hover:text-rose-500 transition-colors p-2"
                    >
                      <Trash2 className="w-5 h-5" />
                    </button>
                  </div>
                </div>
              </div>
            ))}

            {devices.length === 0 && (
              <div className="py-20 text-center bg-white rounded-3xl border border-dashed border-slate-200">
                <Cpu className="w-12 h-12 text-slate-200 mx-auto mb-4" />
                <p className="text-slate-400 font-medium">Chưa có thiết bị nào được thiết lập.</p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
