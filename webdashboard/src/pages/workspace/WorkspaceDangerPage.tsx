import { useState } from "react";

import { AlertTriangle } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/Button";
import Modal from "@/components/ui/Modal";
import PageHeader from "@/components/ui/PageHeader";
import { useToast } from "@/components/ui/Toast";

import { useWorkspace } from "@/hooks/use-workspace";
import { getCurrentWorkspaceId } from "@/lib/api";
import { clearWorkspaceData } from "@/lib/api/workspaces";


export default function WorkspaceDangerPage() {
  const { workspace } = useWorkspace();
  const [isClearDataOpen, setIsClearDataOpen] = useState(false);
  const { t } = useTranslation();

  return (
    <div className="space-y-10 animate-in fade-in duration-500">
      <PageHeader
        title={t('danger.title', { defaultValue: 'Danger Zone' })}
        subtitle={t('danger.subtitle', { defaultValue: 'Destructive operations for this workspace' })}
      />
      
      <div className="bg-rose-50 dark:bg-rose-950/20 p-8 rounded-[2.5rem] border border-rose-200 dark:border-rose-900/50 space-y-6">
        <div className="flex items-center gap-4 border-b border-rose-100 dark:border-rose-900/50 pb-6">
          <div className="w-12 h-12 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center text-rose-500 shadow-sm">
            <AlertTriangle size={24} />
          </div>
          <div>
            <h3 className="text-xl font-bold text-rose-700 dark:text-rose-400">{t('danger.clearDataTitle', { defaultValue: 'Clear Workspace Data' })}</h3>
            <p className="text-sm font-medium text-rose-600/70 dark:text-rose-400/70 mt-1">
              {t('danger.clearDataDesc', { defaultValue: 'Delete all employees, time entries, leaves, and other data. This action cannot be undone.' })}
            </p>
          </div>
        </div>

        <div className="flex justify-end">
          <Button 
            variant="destructive" 
            onClick={() => setIsClearDataOpen(true)}
          >
            {t('danger.clearAllData', { defaultValue: 'CLEAR ALL DATA' })}
          </Button>
        </div>
      </div>

      <ClearDataModal 
        isOpen={isClearDataOpen} 
        onClose={() => setIsClearDataOpen(false)} 
        workspaceName={workspace?.name || ""}
      />
    </div>
  );
}

function ClearDataModal({ isOpen, onClose, workspaceName }: { isOpen: boolean, onClose: () => void, workspaceName: string }) {
  const { t } = useTranslation();
  const [confirmText, setConfirmText] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const toast = useToast();
  const wsId = getCurrentWorkspaceId() || "";

  const handleClear = async () => {
    if (confirmText !== workspaceName) return;
    
    setIsLoading(true);
    try {
      await clearWorkspaceData(wsId, "all");
      toast.success(t('danger.successMessage', { defaultValue: 'Workspace data cleared successfully' }));
      onClose();
      // Reload to reset state
      setTimeout(() => window.location.reload(), 1000);
    } catch (e: any) {
      toast.error(e.message || t('danger.errorMessage', { defaultValue: 'Failed to clear data' }));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={t('danger.modalTitle', { defaultValue: 'Clear Workspace Data' })} icon={<AlertTriangle size={24} />} maxWidth="md">
      <div className="space-y-6 pt-4 animate-in fade-in">
        <div className="p-4 bg-rose-50 dark:bg-rose-950 border border-rose-200 dark:border-rose-900 rounded-2xl text-rose-700 dark:text-rose-300 text-sm font-medium">
          <strong>{t('common.warning', { defaultValue: 'Warning:' })}</strong> {t('danger.warningDesc1', { defaultValue: 'This will permanently delete all data in the workspace, including employees, shifts, time entries, and leaves.' })} 
          <br/>{t('danger.warningDesc2', { defaultValue: 'Members and settings will be preserved.' })}
        </div>

        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">
            {t('danger.typeToConfirmPrefix', { defaultValue: 'Type' })} <strong>{workspaceName}</strong> {t('danger.typeToConfirmSuffix', { defaultValue: 'to confirm' })}
          </label>
          <input 
            type="text" 
            value={confirmText}
            onChange={(e) => setConfirmText(e.target.value)}
            className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold focus:ring-4 focus:ring-rose-500/20 focus:border-rose-500 outline-none transition-all"
            placeholder={workspaceName}
          />
        </div>

        <Button 
          variant="destructive" 
          className="w-full"
          disabled={confirmText !== workspaceName || isLoading}
          loading={isLoading}
          onClick={handleClear}
        >
          {t('danger.permanentlyDelete', { defaultValue: 'PERMANENTLY DELETE DATA' })}
        </Button>
      </div>
    </Modal>
  );
}
