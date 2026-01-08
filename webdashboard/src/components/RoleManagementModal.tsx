import { useState } from 'react';
import { X, Shield, Plus, Trash2, Clock } from 'lucide-react';
import { useMemberRoles } from '@/hooks/use-workspace';

interface RoleManagementModalProps {
  userId: string | null;
  isOpen: boolean;
  onClose: () => void;
}

export default function RoleManagementModal({ userId, isOpen, onClose }: RoleManagementModalProps) {
  const { roles, isLoading, assignRole, revokeRole } = useMemberRoles(userId || '', undefined);
  const [newRole, setNewRole] = useState('member');
  const [expiresAt, setExpiresAt] = useState('');
  
  const primaryRole = roles?.primary || '';
  const extraRoles = roles?.extra || [];

  async function handleAssign(e: React.FormEvent) {
    e.preventDefault();
    if (!expiresAt || !userId) return;
    
    try {
      const isoExpiresAt = new Date(expiresAt).toISOString();
      await assignRole({ role: newRole, expiresAt: isoExpiresAt });
      setExpiresAt('');
    } catch (e) {
      alert('Lỗi khi gán quyền.');
    }
  }

  async function handleRevoke(roleId: string) {
    if (!confirm('Bạn có chắc chắn muốn thu hồi quyền này?')) return;
    try {
      await revokeRole(roleId);
    } catch (e) {
      alert('Lỗi khi thu hồi quyền.');
    }
  }

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/50 backdrop-blur-sm animate-in fade-in duration-200">
      <div className="bg-white w-full max-w-lg rounded-3xl shadow-2xl border border-slate-100 overflow-hidden animate-in zoom-in-95 duration-200">
        <div className="flex items-center justify-between p-6 border-b border-slate-50">
          <div className="flex items-center gap-3">
            <div className="bg-blue-50 p-2 rounded-xl text-blue-600">
              <Shield className="w-5 h-5" />
            </div>
            <h2 className="text-xl font-bold text-slate-900">Quản lý quyền hạn</h2>
          </div>
          <button onClick={onClose} className="p-2 hover:bg-slate-100 rounded-full transition-colors">
            <X className="w-5 h-5 text-slate-400" />
          </button>
        </div>

        <div className="p-6 space-y-8">
          {isLoading ? (
            <div className="py-12 text-center text-slate-400">Đang tải...</div>
          ) : (
            <>
              {/* Primary Role */}
              <div className="space-y-3">
                <label className="text-[10px] font-black uppercase tracking-widest text-slate-400 ml-1">Quyền chính (Vĩnh viễn)</label>
                <div className="flex items-center justify-between p-4 bg-slate-50 rounded-2xl border border-slate-100">
                  <div className="flex items-center gap-3">
                    <Shield className="w-5 h-5 text-blue-600" />
                    <span className="font-bold text-slate-700 capitalize">{primaryRole}</span>
                  </div>
                  <span className="text-[10px] font-bold bg-blue-100 text-blue-600 px-2 py-1 rounded-lg uppercase">Primary</span>
                </div>
              </div>

              {/* Extra Roles */}
              <div className="space-y-4">
                <label className="text-[10px] font-black uppercase tracking-widest text-slate-400 ml-1">Quyền bổ sung (Tạm thời)</label>
                
                <div className="space-y-3">
                  {extraRoles.map((role) => (
                    <div key={role.id} className="flex items-center justify-between p-4 bg-white rounded-2xl border border-slate-100 group hover:border-blue-200 transition-all">
                      <div className="space-y-1">
                        <p className="font-bold text-slate-700 capitalize">{role.role}</p>
                        <div className="flex items-center gap-2 text-xs text-slate-400">
                          <Clock className="w-3 h-3" />
                          <span>Hết hạn: {new Date(role.expires_at).toLocaleString('vi-VN')}</span>
                        </div>
                      </div>
                      <button 
                        onClick={() => handleRevoke(role.id)}
                        className="p-2 text-slate-300 hover:text-rose-600 hover:bg-rose-50 rounded-xl transition-all opacity-0 group-hover:opacity-100"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  ))}

                  {extraRoles.length === 0 && (
                    <div className="text-center py-4 border-2 border-dashed border-slate-100 rounded-2xl text-slate-400 text-sm">
                      Chưa có quyền bổ sung nào.
                    </div>
                  )}
                </div>
              </div>
            </>
          )}

          {/* Assign Form */}
          <div className="pt-6 border-t border-slate-50">
            <h3 className="text-sm font-bold text-slate-800 mb-4 flex items-center gap-2">
              <Plus className="w-4 h-4 text-blue-500" />
              Gán quyền mới
            </h3>
            <form onSubmit={handleAssign} className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-400 uppercase tracking-widest ml-1">Vai trò</label>
                <select 
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 focus:ring-2 focus:ring-blue-500 outline-none transition-all appearance-none text-sm"
                  value={newRole}
                  onChange={e => setNewRole(e.target.value)}
                >
                  <option value="member">Member</option>
                  <option value="admin">Admin</option>
                </select>
              </div>
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-400 uppercase tracking-widest ml-1">Ngày hết hạn</label>
                <div className="relative">
                  <input 
                    required
                    type="datetime-local"
                    className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2 focus:ring-2 focus:ring-blue-500 outline-none transition-all text-sm"
                    value={expiresAt}
                    onChange={e => setExpiresAt(e.target.value)}
                  />
                </div>
              </div>
              <div className="md:col-span-2 pt-2">
                <button type="submit" className="w-full bg-slate-900 text-white py-3 rounded-xl font-bold hover:bg-slate-800 transition-all shadow-lg shadow-slate-200 text-sm">
                  Gán quyền
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}
