import { useState, useEffect } from 'react';
import { ChevronDown, Briefcase, Plus, Check } from 'lucide-react';
import { Workspace, fetchWorkspaces, getCurrentWorkspaceId, setCurrentWorkspaceId } from '@/lib/api';

export default function WorkspaceSwitcher() {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [activeWorkspace, setActiveWorkspace] = useState<Workspace | null>(null);
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    loadWorkspaces();
  }, []);

  async function loadWorkspaces() {
    try {
      const data = await fetchWorkspaces();
      setWorkspaces(data);
      
      const currentId = getCurrentWorkspaceId();
      const current = data.find(ws => ws.id === currentId) || data[0];
      
      if (current) {
        setActiveWorkspace(current);
        if (!currentId) setCurrentWorkspaceId(current.id);
      }
    } catch (e) {
      console.error('Failed to load workspaces', e);
    }
  }

  function handleSwitch(ws: Workspace) {
    setActiveWorkspace(ws);
    setCurrentWorkspaceId(ws.id);
    setIsOpen(false);
    // Reload page to refresh all data with new workspace header
    window.location.reload();
  }

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between gap-3 px-4 py-3 bg-slate-50 hover:bg-slate-100 rounded-2xl transition-all group border border-slate-100"
      >
        <div className="flex items-center gap-3 overflow-hidden">
          <div className="bg-white p-1.5 rounded-lg shadow-sm">
            <Briefcase className="w-4 h-4 text-blue-600" />
          </div>
          <span className="font-bold text-sm text-slate-700 truncate">
            {activeWorkspace?.name || 'Chọn Workspace'}
          </span>
        </div>
        <ChevronDown className={`w-4 h-4 text-slate-400 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
      </button>

      {isOpen && (
        <>
          <div 
            className="fixed inset-0 z-10" 
            onClick={() => setIsOpen(false)}
          />
          <div className="absolute top-full left-0 right-0 mt-2 bg-white rounded-2xl shadow-xl border border-slate-100 py-2 z-20 animate-in fade-in slide-in-from-top-2 duration-200">
            <div className="px-4 py-2 mb-1">
              <span className="text-[10px] font-black uppercase tracking-widest text-slate-400">Workspaces của bạn</span>
            </div>
            
            <div className="max-h-60 overflow-y-auto">
              {workspaces.map((ws) => (
                <button
                  key={ws.id}
                  onClick={() => handleSwitch(ws)}
                  className="w-full flex items-center justify-between px-4 py-2.5 hover:bg-slate-50 transition-colors text-left group"
                >
                  <span className={`text-sm font-semibold ${ws.id === activeWorkspace?.id ? 'text-blue-600' : 'text-slate-600'}`}>
                    {ws.name}
                  </span>
                  {ws.id === activeWorkspace?.id && (
                    <Check className="w-4 h-4 text-blue-600" />
                  )}
                </button>
              ))}
            </div>

            <div className="border-t border-slate-50 mt-2 pt-2 px-2">
              <button className="w-full flex items-center gap-2 px-3 py-2 text-slate-500 hover:text-blue-600 hover:bg-blue-50 rounded-xl transition-all text-sm font-bold group">
                <Plus className="w-4 h-4 group-hover:scale-110 transition-transform" />
                Thêm Workspace
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
