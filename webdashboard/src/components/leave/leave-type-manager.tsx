import { Wallet, AlertTriangle, Pencil, Plus, Trash2, Sparkles } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { LeaveType, LeaveTypeConfig } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { useLeaveTypeManager } from "./hooks/use-leave-type-manager";
import { LeaveTypeForm } from "./leave-policy-components/leave-type-form";
import { TemplateGallery } from "./leave-policy-components/template-gallery";
import { LeaveTemplate } from "./leave-templates";

interface LeaveTypeManagerProps {
  wsId: string;
}

// eslint-disable-next-line max-lines-per-function
export function LeaveTypeManager({ wsId }: LeaveTypeManagerProps) {
  const { t } = useTranslation();
  const {
    editingType,
    isAddingType,
    deletingType,
    leaveTypes,
    isTypesLoading,
    setEditingType,
    setIsAddingType,
    setDeletingType,
    createLeaveType,
    updateLeaveType,
    deleteLeaveType: triggerDelete,
    isCreating,
    isUpdating,
    isDeleting,
  } = useLeaveTypeManager({ wsId });

  if (isTypesLoading) {
    return (
      <div className="py-12 text-center text-slate-400 dark:text-slate-500 animate-pulse font-bold uppercase tracking-widest text-[10px]">
        {t("settings.leave.type_manager.loading", "Loading leave types...")}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center px-1">
        <Typography className="text-xs font-medium text-slate-500 dark:text-slate-400 max-w-md italic" variant="p">
          {t("settings.leave.type_manager.desc", "Define the categories of leave available in your organization (e.g., Annual Leave, Sick Leave).")}
        </Typography>
        <Button
          className="shadow-primary-100 text-[10px]"
          onClick={() => {
            setIsAddingType(true);
          }}
          size="sm"
          variant="primary"
        >
          <Plus size={14} /> {t("settings.leave.type_manager.add_btn", "Add Leave Type")}
        </Button>
      </div>

      {leaveTypes.length > 0 ? (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {leaveTypes.map((type) => (
            <div
              className="relative flex flex-col justify-between rounded-[2rem] border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-7 shadow-sm hover:shadow-md transition-all group"
              key={type.id}
            >
              <div>
                <div className="flex items-center justify-between">
                  <div
                    className={cn(
                      "flex h-12 w-12 items-center justify-center rounded-2xl shadow-inner",
                      type.is_paid
                        ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border border-emerald-100 dark:border-emerald-800"
                        : "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border border-amber-100 dark:border-amber-800"
                    )}
                  >
                    {type.is_paid ? <Wallet size={20} /> : <AlertTriangle size={20} />}
                  </div>
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <Button
                      className="h-9 w-9 p-2 text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:text-primary-400 bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800"
                      onClick={() => {
                        setEditingType(type);
                      }}
                      size="icon"
                      variant="outline"
                    >
                      <Pencil size={16} />
                    </Button>
                    <Button
                      className="h-9 w-9 p-2 text-slate-400 dark:text-slate-500 hover:text-rose-600 dark:text-rose-300 bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800"
                      onClick={() => {
                        setDeletingType(type);
                      }}
                      size="icon"
                      variant="outline"
                    >
                      <Trash2 size={16} />
                    </Button>
                  </div>
                </div>
                <Typography className="mt-5 text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight leading-none uppercase" variant="h3">
                  {t(`settings.leave.templates.${type.code.toLowerCase()}.name`, type.name)}
                </Typography>
                <Typography className="mt-1 text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
                  {t("settings.leave.type_manager.code", "Code")}: {type.code}
                </Typography>
                <div className="mt-4 flex flex-wrap gap-2">
                  <span
                    className={cn(
                      "px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border",
                      type.is_paid
                        ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
                        : "bg-amber-50 dark:bg-amber-950 text-amber-700 dark:text-amber-300 border-amber-100 dark:border-amber-800"
                    )}
                  >
                    {type.is_paid ? t("settings.leave.type_manager.paid", "Paid") : t("settings.leave.type_manager.unpaid", "Unpaid")}
                  </span>
                  {!!type.policy_config?.approval_required && (
                    <span className="px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 border-blue-100 dark:border-blue-800">
                      {t("settings.leave.fields.approval", "Approval")}
                    </span>
                  )}
                  {!!type.policy_config?.requires_document && (
                    <span className="px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border bg-purple-50 dark:bg-purple-950 text-purple-700 dark:text-purple-300 border-purple-100 dark:border-purple-800">
                      {t("settings.leave.fields.document", "Document")}
                    </span>
                  )}
                  {!!type.policy_config?.allow_negative_balance && (
                    <span className="px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border bg-rose-50 dark:bg-rose-950 text-rose-700 dark:text-rose-300 border-rose-100 dark:border-rose-800">
                      {t("settings.leave.type_manager.negative_balance", "Negative Balance")}
                    </span>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <EmptyState
          action={
            <Button
              className="shadow-primary-200 dark:shadow-none text-[10px]"
              onClick={() => {
                setIsAddingType(true);
              }}
              size="md"
              variant="primary"
            >
              <Plus size={16} />
              {t("settings.leave.type_manager.get_started", "Get Started")}
            </Button>
          }
          description={t("settings.leave.type_manager.empty_desc", "Your workspace is fresh and clean. Start by creating leave types manually or use our professional templates.")}
          icon={<Sparkles className="text-primary-400" size={40} />}
          title={t("settings.leave.type_manager.empty_title", "No Leave Types Configured")}
        />
      )}

      {isAddingType ? (
        <AddLeaveTypeModal
          existingCodes={leaveTypes.map((lt) => lt.code.toLowerCase())}
          isSaving={isCreating}
          onClose={() => {
            setIsAddingType(false);
          }}
          onSave={createLeaveType}
        />
      ) : null}

      {editingType ? (
        <EditLeaveTypeModal
          isSaving={isUpdating}
          leaveType={editingType}
          onClose={() => {
            setEditingType(null);
          }}
          onSave={(config: LeaveTypeConfig) => {
            updateLeaveType(editingType.id, config);
          }}
        />
      ) : null}

      {deletingType ? (
        <Modal
          isOpen={true}
          onClose={() => {
            setDeletingType(null);
          }}
          title={t("settings.leave.type_manager.confirm_delete_title", "Confirm Delete")}
        >
          <div className="p-8">
            <Typography className="text-sm" variant="p">
              {t("settings.leave.type_manager.confirm_delete_msg", "Are you sure you want to delete \"" + deletingType.name + "\"?", { name: deletingType.name })}
            </Typography>
            <div className="mt-8 flex justify-end gap-3">
              <Button
                className="px-6 py-2 text-xs text-slate-400 dark:text-slate-500 hover:bg-transparent"
                onClick={() => {
                  setDeletingType(null);
                }}
                variant="ghost"
              >
                {t("common.cancel", "Cancel")}
              </Button>
              <Button
                className="text-[10px] px-6 py-2.5 h-auto rounded-xl"
                disabled={isDeleting}
                onClick={() => {
                  triggerDelete(deletingType.id);
                }}
                variant="destructive"
              >
                {t("settings.leave.type_manager.delete_btn", "Delete")}
              </Button>
            </div>
          </div>
        </Modal>
      ) : null}
    </div>
  );
}

interface AddLeaveTypeModalProps {
  existingCodes: string[];
  isSaving: boolean;
  onClose: () => void;
  onSave: (data: { code: string; name: string; is_paid: boolean }) => void;
}

function AddLeaveTypeModal({ existingCodes, onClose, onSave, isSaving }: AddLeaveTypeModalProps) {
  const { t } = useTranslation();
  const [step, setStep] = useState<"template" | "form">("template");
  const [formData, setFormData] = useState({ code: "", name: "", is_paid: true });

  // Proactive inline duplicate detection (matches the workspace-unique,
  // case-insensitive code constraint enforced server-side).
  const normalizedCode = formData.code.trim().toLowerCase();
  const isDuplicateCode =
    normalizedCode.length > 0 && existingCodes.includes(normalizedCode);

  const handleApplyTemplate = (template: LeaveTemplate) => {
    setFormData({
      code: template.code,
      name: template.name,
      is_paid: template.is_paid,
    });
    setStep("form");
  };

  return (
    <Modal
      isOpen={true}
      maxWidth={step === "template" ? "2xl" : "md"}
      onClose={onClose}
      subtitle={
        step === "template"
          ? t("settings.leave.gallery.subtitle", "Start faster with a professional setup")
          : t("settings.leave.gallery.customize_subtitle", "Refining " + (formData.name || t("settings.leave.gallery.customize", "Customize Leave Type")), { name: formData.name || t("settings.leave.gallery.customize", "Customize Leave Type") })
      }
      title={step === "template" ? t("settings.leave.gallery.title", "Choose a Template") : t("settings.leave.gallery.customize", "Customize Leave Type")}
    >
      {step === "template" ? (
        <TemplateGallery
          onSelectTemplate={handleApplyTemplate}
          onSkip={() => {
            setStep("form");
          }}
        />
      ) : (
        <LeaveTypeForm
          existingCodes={existingCodes}
          initialData={formData}
          isSaving={isSaving}
          onBackToTemplates={() => {
            setStep("template");
          }}
          onCancel={onClose}
          onSave={onSave}
        />
      )}
    </Modal>
  );
}

interface EditLeaveTypeModalProps {
  isSaving: boolean;
  leaveType: LeaveType;
  onClose: () => void;
  onSave: (config: LeaveTypeConfig) => void;
}

 
function EditLeaveTypeModal({ leaveType, onClose, onSave, isSaving }: EditLeaveTypeModalProps) {
  const { t } = useTranslation();
  const [config, setConfig] = useState(() => ({
    approval_required: true,
    requires_document: false,
    allow_negative_balance: false,
    ...leaveType.policy_config,
  }));

  return (
    <Modal
      isOpen={true}
      maxWidth="md"
      onClose={onClose}
      subtitle={t("settings.leave.type_manager.policy_config_subtitle", "Defining rules for " + leaveType.name, { name: leaveType.name })}
      title={t("settings.leave.type_manager.policy_config_title", "Leave Policy Configuration")}
    >
      <form
        className="space-y-6"
        onSubmit={(e) => {
          e.preventDefault();
          onSave(config);
        }}
      >
        <div className="rounded-[2rem] bg-slate-50/50 dark:bg-slate-800/30 border border-slate-100 dark:border-slate-800 overflow-hidden">
          <div className="flex items-center justify-between p-4 border-b border-slate-100 dark:border-slate-800 bg-slate-50/30 dark:bg-slate-900/10">
            <div className="flex items-center gap-3">
              <div
                className={cn(
                  "h-9 w-9 rounded-xl flex items-center justify-center shadow-inner shrink-0",
                  leaveType.is_paid
                    ? "bg-emerald-50 dark:bg-emerald-950/50 text-emerald-600"
                    : "bg-amber-50 dark:bg-amber-950/50 text-amber-600"
                )}
              >
                {leaveType.is_paid ? <Wallet size={16} /> : <AlertTriangle size={16} />}
              </div>
              <div>
                <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block">
                  {t("settings.leave.type_form.paid_status", "Paid Status")}
                </span>
                <Typography className="text-[10px] text-slate-400 dark:text-slate-500" variant="p">
                  {leaveType.is_paid 
                    ? t("settings.leave.type_form.paid_time_off", "Paid time off") 
                    : t("settings.leave.type_form.unpaid_leave", "Unpaid leave")}
                </Typography>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <span
                className={cn(
                  "inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider border",
                  leaveType.is_paid
                    ? "bg-emerald-50/50 dark:bg-emerald-950/20 text-emerald-600 dark:text-emerald-400 border-emerald-100 dark:border-emerald-900/50"
                    : "bg-amber-50/50 dark:bg-amber-950/20 text-amber-600 dark:text-amber-400 border-amber-100 dark:border-amber-900/50"
                )}
              >
                {leaveType.is_paid 
                  ? t("settings.leave.type_manager.paid", "Paid") 
                  : t("settings.leave.type_manager.unpaid", "Unpaid")}
              </span>
              <span className="text-[9px] text-slate-400 dark:text-slate-500 italic">
                ({t("settings.leave.type_manager.immutable", "Non-editable")})
              </span>
            </div>
          </div>

          <div className="flex items-center justify-between p-4 border-b border-slate-100 dark:border-slate-800">
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-xl bg-primary-50 dark:bg-primary-950/50 text-primary-600 flex items-center justify-center shadow-inner shrink-0">
                <Plus size={16} />
              </div>
              <div>
                <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block">
                  {t("settings.leave.type_manager.approval_required", "Approval Required")}
                </span>
                <Typography className="text-[10px] text-slate-400 dark:text-slate-500" variant="p">{t("settings.leave.type_manager.approval_required_desc", "Managers must approve requests")}</Typography>
              </div>
            </div>
            <div className="relative inline-flex items-center">
              <input
                checked={config.approval_required}
                className="sr-only peer"
                id="approval-req-input"
                onChange={(e) => {
                  setConfig({ ...config, approval_required: e.target.checked });
                }}
                type="checkbox"
              />
              <label
                className="w-10 h-5.5 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4.5 after:w-4.5 after:transition-all peer-checked:bg-primary-600 cursor-pointer"
                htmlFor="approval-req-input"
              >
                <span className="sr-only">{t("settings.leave.type_manager.approval_required", "Approval Required")}</span>
              </label>
            </div>
          </div>

          <div className="flex items-center justify-between p-4 border-b border-slate-100 dark:border-slate-800">
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-xl bg-blue-50 dark:bg-blue-950/50 text-blue-600 flex items-center justify-center shadow-inner shrink-0">
                <Sparkles size={16} />
              </div>
              <div>
                <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block">
                  {t("settings.leave.type_manager.requires_document", "Requires Document")}
                </span>
                <Typography className="text-[10px] text-slate-400 dark:text-slate-500" variant="p">{t("settings.leave.type_manager.requires_document_desc", "Proof/Medical cert is mandatory")}</Typography>
              </div>
            </div>
            <div className="relative inline-flex items-center">
              <input
                checked={config.requires_document}
                className="sr-only peer"
                id="doc-req-input"
                onChange={(e) => {
                  setConfig({ ...config, requires_document: e.target.checked });
                }}
                type="checkbox"
              />
              <label
                className="w-10 h-5.5 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4.5 after:w-4.5 after:transition-all peer-checked:bg-primary-600 cursor-pointer"
                htmlFor="doc-req-input"
              >
                <span className="sr-only">{t("settings.leave.type_manager.requires_document", "Requires Document")}</span>
              </label>
            </div>
          </div>

          <div className="flex items-center justify-between p-4 flex-wrap sm:flex-nowrap">
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-xl bg-amber-50 dark:bg-amber-950/50 text-amber-600 flex items-center justify-center shadow-inner shrink-0">
                <AlertTriangle size={16} />
              </div>
              <div>
                <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block">
                  {t("settings.leave.type_manager.negative_balance", "Negative Balance")}
                </span>
                <Typography className="text-[10px] text-slate-400 dark:text-slate-500" variant="p">{t("settings.leave.type_manager.negative_balance_desc", "Allow users to take leave beyond 0")}</Typography>
              </div>
            </div>
            <div className="relative inline-flex items-center">
              <input
                checked={config.allow_negative_balance}
                className="sr-only peer"
                id="negative-bal-input"
                onChange={(e) => {
                  setConfig({ ...config, allow_negative_balance: e.target.checked });
                }}
                type="checkbox"
              />
              <label
                className="w-10 h-5.5 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4.5 after:w-4.5 after:transition-all peer-checked:bg-primary-600 cursor-pointer"
                htmlFor="negative-bal-input"
              >
                <span className="sr-only">{t("settings.leave.type_manager.negative_balance", "Negative Balance")}</span>
              </label>
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-3 pt-6 border-t border-slate-50 dark:border-slate-800">
          <Button
            className="px-6 py-3 text-[10px] text-slate-400 dark:text-slate-500 hover:text-slate-600"
            onClick={onClose}
            size="sm"
            type="button"
            variant="ghost"
          >
            {t("common.cancel", "Cancel")}
          </Button>
          <Button
            className="shadow-primary-500/20 text-[10px] tracking-[0.2em] py-3.5 px-10"
            disabled={isSaving}
            size="sm"
            type="submit"
            variant="primary"
          >
            {isSaving ? t("settings.leave.type_manager.saving", "Saving...") : t("settings.leave.type_manager.save_policy", "Save Policy")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
