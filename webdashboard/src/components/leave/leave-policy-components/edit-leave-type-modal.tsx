import { AlertTriangle, Plus, Sparkles, Wallet } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { LeaveType } from "@/lib/api/types";

type LeaveTypeConfig = { approval_required: boolean; requires_document: boolean; allow_negative_balance: boolean };
import { cn } from "@/lib/utils";

interface ToggleRowProps {
  description: string;
  icon: React.ReactNode;
  id: string;
  label: string;
  onChange: (checked: boolean) => void;
  value: boolean;
}

function ToggleRow({ icon, label, description, id, value, onChange }: ToggleRowProps) {
  return (
    <div className="flex items-center justify-between p-4 border-b border-slate-100 dark:border-slate-800">
      <div className="flex items-center gap-3">
        <div className="h-9 w-9 rounded-xl bg-primary-50 dark:bg-primary-950/50 text-primary-600 flex items-center justify-center shadow-inner shrink-0">
          {icon}
        </div>
        <div>
          <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block">
            {label}
          </span>
          <Typography className="text-[10px] text-slate-400 dark:text-slate-500" variant="p">{description}</Typography>
        </div>
      </div>
      <div className="relative inline-flex items-center">
        <input
          checked={value}
          className="sr-only peer"
          id={id}
          onChange={(e) => { onChange(e.target.checked); }}
          type="checkbox"
        />
        <label
          className="w-10 h-5.5 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4.5 after:w-4.5 after:transition-all peer-checked:bg-primary-600 cursor-pointer"
          htmlFor={id}
        >
          <span className="sr-only">{label}</span>
        </label>
      </div>
    </div>
  );
}

interface EditLeaveTypeModalProps {
  isSaving: boolean;
  leaveType: LeaveType;
  onClose: () => void;
  onSave: (config: LeaveTypeConfig) => void;
}

export default function EditLeaveTypeModal({ leaveType, onClose, onSave, isSaving }: EditLeaveTypeModalProps) {
  const { t } = useTranslation();
  const [config, setConfig] = useState<LeaveTypeConfig>(() => ({
    approval_required: true,
    requires_document: false,
    allow_negative_balance: false,
    ...(leaveType.policy_config && typeof leaveType.policy_config === "object" && !Array.isArray(leaveType.policy_config) ? leaveType.policy_config : {}),
  }));

  const setField = (field: keyof typeof config) => (value: boolean) => {
    setConfig((prev) => ({ ...prev, [field]: value }));
  };

  return (
    <Modal
      isOpen
      maxWidth="md"
      onClose={onClose}
      subtitle={t("settings.leave.type_manager.policy_config_subtitle", "Defining rules for " + leaveType.name, { name: leaveType.name })}
      title={t("settings.leave.type_manager.policy_config_title", "Leave Policy Configuration")}
    >
      <form className="space-y-6" onSubmit={(e) => { e.preventDefault(); onSave(config); }}>
        <div className="rounded-[2rem] bg-slate-50/50 dark:bg-slate-800/30 border border-slate-100 dark:border-slate-800 overflow-hidden">
          {/* Paid Status (immutable) */}
          <div className="flex items-center justify-between p-4 border-b border-slate-100 dark:border-slate-800 bg-slate-50/30 dark:bg-slate-900/10">
            <div className="flex items-center gap-3">
              <div className={cn("h-9 w-9 rounded-xl flex items-center justify-center shadow-inner shrink-0",
                leaveType.is_paid ? "bg-emerald-50 dark:bg-emerald-950/50 text-emerald-600" : "bg-amber-50 dark:bg-amber-950/50 text-amber-600")}>
                {leaveType.is_paid ? <Wallet size={16} /> : <AlertTriangle size={16} />}
              </div>
              <div>
                <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block">
                  {t("settings.leave.type_form.paid_status")}
                </span>
                <Typography className="text-[10px] text-slate-400 dark:text-slate-500" variant="p">
                  {leaveType.is_paid
                    ? t("settings.leave.type_form.paid_time_off")
                    : t("settings.leave.type_form.unpaid_leave")}
                </Typography>
              </div>
            </div>
            <span className={cn("inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider border",
              leaveType.is_paid
                ? "bg-emerald-50/50 dark:bg-emerald-950/20 text-emerald-600 dark:text-emerald-400 border-emerald-100 dark:border-emerald-900/50"
                : "bg-amber-50/50 dark:bg-amber-950/20 text-amber-600 dark:text-amber-400 border-amber-100 dark:border-amber-900/50")}>
              {leaveType.is_paid ? t("settings.leave.type_manager.paid") : t("settings.leave.type_manager.unpaid")}
            </span>
          </div>

          <ToggleRow
            description={t("settings.leave.type_manager.approval_required_desc", "Managers must approve requests")}
            icon={<Plus size={16} />}
            id="approval-req-input"
            label={t("settings.leave.type_manager.approval_required", "Approval Required")}
            onChange={setField("approval_required")}
            value={config.approval_required}
          />
          <ToggleRow
            description={t("settings.leave.type_manager.requires_document_desc", "Proof/Medical cert is mandatory")}
            icon={<Sparkles size={16} />}
            id="doc-req-input"
            label={t("settings.leave.type_manager.requires_document", "Requires Document")}
            onChange={setField("requires_document")}
            value={config.requires_document}
          />
          <ToggleRow
            description={t("settings.leave.type_manager.negative_balance_desc", "Allow users to take leave beyond 0")}
            icon={<AlertTriangle size={16} />}
            id="negative-bal-input"
            label={t("settings.leave.type_manager.negative_balance", "Negative Balance")}
            onChange={setField("allow_negative_balance")}
            value={config.allow_negative_balance}
          />
        </div>

        <div className="flex justify-end gap-3 pt-6 border-t border-slate-50 dark:border-slate-800">
          <Button className="px-6 py-3 text-[10px] text-slate-400 dark:text-slate-500" onClick={onClose} size="sm" type="button" variant="ghost">
            {t("common.cancel", "Cancel")}
          </Button>
          <Button className="shadow-primary-500/20 text-[10px] tracking-[0.2em] py-3.5 px-10" disabled={isSaving} size="sm" type="submit" variant="primary">
            {isSaving ? t("settings.leave.type_manager.saving") : t("settings.leave.type_manager.save_policy")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
