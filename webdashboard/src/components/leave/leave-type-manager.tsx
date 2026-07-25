/* eslint-disable @typescript-eslint/no-unnecessary-condition -- leave type config fields may be null at runtime */
import { Wallet, AlertTriangle, Pencil, Plus, Sparkles, Trash2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { LeaveType } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { useLeaveTypeManager } from "./hooks/use-leave-type-manager";
import EditLeaveTypeModal from "./leave-policy-components/edit-leave-type-modal";
import { LeaveTypeForm } from "./leave-policy-components/leave-type-form";
import { TemplateGallery } from "./leave-policy-components/template-gallery";
import { LeaveTemplate } from "./leave-templates";

// ── PolicyBadges ─────────────────────────────────────────────────────

function PolicyBadges({ type, t }: { type: LeaveType; t: (k: string) => string }) {
  return (
    <>
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
    </>
  );
}

// ── LeaveTypeCard ────────────────────────────────────────────────────

function LeaveTypeCard({ type, onEdit, onDelete }: {
  type: LeaveType;
  onEdit: (t: LeaveType) => void;
  onDelete: (t: LeaveType) => void;
}) {
  const { t } = useTranslation();

  const paidClasses = type.is_paid
    ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border border-emerald-100 dark:border-emerald-800"
    : "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border border-amber-100 dark:border-amber-800";

  const paidBadgeClasses = type.is_paid
    ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
    : "bg-amber-50 dark:bg-amber-950 text-amber-700 dark:text-amber-300 border-amber-100 dark:border-amber-800";

  return (
    <div className="relative flex flex-col justify-between rounded-[2rem] border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-7 shadow-sm hover:shadow-md transition-all group">
      <div>
        <div className="flex items-center justify-between">
          <div className={cn("flex h-12 w-12 items-center justify-center rounded-2xl shadow-inner", paidClasses)}>
            {type.is_paid ? <Wallet size={20} /> : <AlertTriangle size={20} />}
          </div>
          <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <Button
              className="h-9 w-9 p-2 text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:text-primary-400 bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800"
              onClick={() => { onEdit(type); }}
              size="icon"
              variant="outline"
            >
              <Pencil size={16} />
            </Button>
            <Button
              className="h-9 w-9 p-2 text-slate-400 dark:text-slate-500 hover:text-rose-600 dark:text-rose-300 bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800"
              onClick={() => { onDelete(type); }}
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
          <span className={cn("px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border", paidBadgeClasses)}>
            {type.is_paid ? t("settings.leave.type_manager.paid", "Paid") : t("settings.leave.type_manager.unpaid", "Unpaid")}
          </span>
          <PolicyBadges t={t} type={type} />
        </div>
      </div>
    </div>
  );
}

interface LeaveTypeManagerProps {
  wsId: string;
}

export function LeaveTypeManager({ wsId }: LeaveTypeManagerProps) {
  const { t } = useTranslation();
  const {
    editingType,
    isAddingType,
    deletingType,
    restoringType,
    leaveTypes,
    isTypesLoading,
    setEditingType,
    setIsAddingType,
    setDeletingType,
    setRestoringType,
    createLeaveType,
    updateLeaveType,
    deleteLeaveType: triggerDelete,
    restoreLeaveType: triggerRestore,
    isCreating,
    isUpdating,
    isDeleting,
    isRestoring,
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
            <LeaveTypeCard
              key={type.id}
              onDelete={(t) => { setDeletingType(t); }}
              onEdit={(t) => { setEditingType(t); }}
              type={type}
            />
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
          onClose={() => { setIsAddingType(false); }}
          onSave={createLeaveType}
        />
      ) : null}

      {editingType ? (
        <EditLeaveTypeModal
          isSaving={isUpdating}
          leaveType={editingType}
          onClose={() => { setEditingType(null); }}
          onSave={(config) => { updateLeaveType(editingType.id, config); }}
        />
      ) : null}

      <ConfirmDeleteModal
        isDeleting={isDeleting}
        name={deletingType?.name ?? ""}
        onCancel={() => { setDeletingType(null); }}
        onConfirm={() => { if (deletingType) triggerDelete(deletingType.id); }}
        open={deletingType !== null}
        t={t}
      />

      <RestoreTypeModal
        isRestoring={isRestoring}
        name={restoringType?.name ?? ""}
        onCancel={() => { setRestoringType(null); }}
        onConfirm={() => { if (restoringType) triggerRestore(restoringType.leave_type_id); }}
        open={restoringType !== null}
        t={t}
      />
    </div>
  );
}

function ConfirmDeleteModal({
  open,
  name,
  isDeleting,
  onCancel,
  onConfirm,
  t,
}: {
  open: boolean;
  name: string;
  isDeleting: boolean;
  onCancel: () => void;
  onConfirm: () => void;
  t: ReturnType<typeof useTranslation>["t"];
}) {
  if (!open) return null;
  return (
    <Modal
      isOpen={true}
      onClose={onCancel}
      title={t("settings.leave.type_manager.confirm_delete_title", "Confirm Delete")}
    >
      <div className="p-8">
        <Typography className="text-sm" variant="p">
          {t("settings.leave.type_manager.confirm_delete_msg", "Are you sure you want to delete \"{{name}}\"?", { name })}
        </Typography>
        <div className="mt-8 flex justify-end gap-3">
          <Button
            className="px-6 py-2 text-xs text-slate-400 dark:text-slate-500 hover:bg-transparent"
            onClick={onCancel}
            variant="ghost"
          >
            {t("common.cancel", "Cancel")}
          </Button>
          <Button
            className="text-[10px] px-6 py-2.5 h-auto rounded-xl"
            disabled={isDeleting}
            onClick={onConfirm}
            variant="destructive"
          >
            {t("settings.leave.type_manager.delete_btn", "Delete")}
          </Button>
        </div>
      </div>
    </Modal>
  );
}

function RestoreTypeModal({
  open,
  name,
  isRestoring,
  onCancel,
  onConfirm,
  t,
}: {
  open: boolean;
  name: string;
  isRestoring: boolean;
  onCancel: () => void;
  onConfirm: () => void;
  t: ReturnType<typeof useTranslation>["t"];
}) {
  if (!open) return null;
  return (
    <Modal
      isOpen={true}
      onClose={onCancel}
      title={t("settings.leave.type_manager.restore_dialog_title", "Restore Leave Type")}
    >
      <div className="p-8">
        <Typography className="text-sm" variant="p">
          {t(
            "settings.leave.type_manager.restore_dialog_msg",
            "A leave type with this code was retired. Restore \"{{name}}\" instead of creating a new one?",
            { name },
          )}
        </Typography>
        <div className="mt-8 flex justify-end gap-3">
          <Button
            className="px-6 py-2 text-xs text-slate-400 dark:text-slate-500 hover:bg-transparent"
            onClick={onCancel}
            variant="ghost"
          >
            {t("common.cancel", "Cancel")}
          </Button>
          <Button
            className="text-[10px] px-6 py-2.5 h-auto rounded-xl"
            disabled={isRestoring}
            onClick={onConfirm}
            variant="primary"
          >
            {t("settings.leave.type_manager.restore_btn", "Restore")}
          </Button>
        </div>
      </div>
    </Modal>
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
  const _isDuplicateCode =
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

// EditLeaveTypeModal extracted to ./leave-policy-components/edit-leave-type-modal.tsx
