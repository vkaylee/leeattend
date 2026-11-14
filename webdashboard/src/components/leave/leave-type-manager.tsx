import { useNavigate } from "@tanstack/react-router";
import { Wallet, AlertTriangle, Pencil, Plus, Sparkles, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { LeaveType } from "@/lib/api/types";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

import { useLeaveTypeManager } from "./hooks/use-leave-type-manager";
import EditLeaveTypeModal from "./leave-policy-components/edit-leave-type-modal";
import { RestoreTypeModal } from "./leave-policy-components/restore-type-modal";

// ── PolicyBadges ─────────────────────────────────────────────────────

function PolicyBadges({ type, t }: { type: LeaveType; t: ((k: string, d?: string) => string) | ((k: string) => string) }) {
  // Generated types say policy_config is non-null, but API may omit it at runtime
  const policy = type.policy_config as LeaveType["policy_config"] | null | undefined;
  const hasPolicy = !!policy;
  return (
    <>
      {hasPolicy && policy.approval_required ? (
        <span className="px-2.5 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 border-blue-100 dark:border-blue-800">
          {t("settings.leave.fields.approval", "Approval")}
        </span>
      ) : null}
      {hasPolicy && policy.requires_document ? (
        <span className="px-2.5 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-purple-50 dark:bg-purple-950 text-purple-700 dark:text-purple-300 border-purple-100 dark:border-purple-800">
          {t("settings.leave.fields.document", "Document")}
        </span>
      ) : null}
      {hasPolicy && policy.allow_negative_balance ? (
        <span className="px-2.5 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-rose-50 dark:bg-rose-950 text-rose-700 dark:text-rose-300 border-rose-100 dark:border-rose-800">
          {t("settings.leave.type_manager.negative_balance", "Negative Balance")}
        </span>
      ) : null}
    </>
  );
}

// ── LeaveTypeCard ────────────────────────────────────────────────────

export function LeaveTypeCard({ type, onEdit, onDelete }: {
  type: LeaveType;
  onDelete?: (t: LeaveType) => void;
  onEdit?: (t: LeaveType) => void;
}) {
  const { t } = useTranslation();

  const paidClasses = type.is_paid
    ? "bg-emerald-50 dark:bg-emerald-950/70 text-emerald-600 dark:text-emerald-300 border border-emerald-200/60 dark:border-emerald-800/60 shadow-xs"
    : "bg-amber-50 dark:bg-amber-950/70 text-amber-600 dark:text-amber-300 border border-amber-200/60 dark:border-amber-800/60 shadow-xs";

  const paidBadgeClasses = type.is_paid
    ? "bg-emerald-50 dark:bg-emerald-950/70 text-emerald-700 dark:text-emerald-300 border-emerald-200/70 dark:border-emerald-800/70"
    : "bg-amber-50 dark:bg-amber-950/70 text-amber-700 dark:text-amber-300 border-amber-200/70 dark:border-amber-800/70";

  const hasActions = Boolean(onEdit) || Boolean(onDelete);

  return (
    <div className="relative flex flex-col justify-between rounded-3xl border border-slate-200/90 dark:border-slate-800 bg-white dark:bg-slate-900/90 p-7 shadow-2xs hover:shadow-xl hover:shadow-slate-200/50 dark:hover:shadow-black/40 hover:-translate-y-1 transition-all duration-300 group overflow-hidden">
      <div className="absolute top-0 right-0 w-36 h-36 bg-gradient-to-br from-primary-500/5 to-transparent rounded-full blur-2xl pointer-events-none group-hover:scale-150 transition-transform duration-500" />
      <div>
        <div className="flex items-center justify-between">
          <div className={cn("flex h-12 w-12 items-center justify-center rounded-2xl shadow-inner transition-transform group-hover:scale-105 duration-200", paidClasses)}>
            {type.is_paid ? <Wallet size={20} /> : <AlertTriangle size={20} />}
          </div>
          {hasActions ? (
            <div className="flex gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
              {onEdit ? (
                <Button
                  aria-label={t("common.edit", "Edit")}
                  className="h-9 w-9 p-2 text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:text-primary-400 bg-slate-50 dark:bg-slate-800 border-slate-200/60 dark:border-slate-700 rounded-xl"
                  onClick={() => { onEdit(type); }}
                  size="icon"
                  variant="outline"
                >
                  <Pencil size={15} />
                </Button>
              ) : null}
              {onDelete ? (
                <Button
                  aria-label={t("common.delete", "Delete")}
                  className="h-9 w-9 p-2 text-slate-400 dark:text-slate-500 hover:text-rose-600 dark:text-rose-300 bg-slate-50 dark:bg-slate-800 border-slate-200/60 dark:border-slate-700 rounded-xl"
                  onClick={() => { onDelete(type); }}
                  size="icon"
                  variant="outline"
                >
                  <Trash2 size={15} />
                </Button>
              ) : null}
            </div>
          ) : null}
        </div>
        <Typography className="mt-5 text-lg font-bold text-slate-900 dark:text-slate-100 tracking-tight leading-none uppercase" variant="h3">
          {t(`settings.leave.templates.${type.code.toLowerCase()}.name`, type.name)}
        </Typography>
        <Typography className="mt-1 text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
          {t("settings.leave.type_manager.code", "Code")}: {type.code}
        </Typography>
        <div className="mt-4 flex flex-wrap gap-2">
          <span className={cn("px-2.5 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border flex items-center gap-1.5", paidBadgeClasses)}>
            <span className={cn("w-1.5 h-1.5 rounded-full", type.is_paid ? "bg-emerald-500" : "bg-amber-500")} />
            {type.is_paid ? t("settings.leave.type_manager.paid", "Paid") : t("settings.leave.type_manager.unpaid", "Unpaid")}
          </span>
          <PolicyBadges t={t} type={type} />
        </div>
      </div>
    </div>
  );
}

export function LeaveTypeManager() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const {
    editingType,
    deletingType,
    restoringType,
    leaveTypes,
    isTypesLoading,
    setEditingType,
    setDeletingType,
    setRestoringType,
    updateLeaveType,
    deleteLeaveType: triggerDelete,
    restoreLeaveType: triggerRestore,
    isUpdating,
    isDeleting,
    isRestoring,
  } = useLeaveTypeManager();

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
            void navigate({ to: ROUTES.SETTINGS.LEAVE_TYPES_NEW });
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
                void navigate({ to: ROUTES.SETTINGS.LEAVE_TYPES_NEW });
              }}
              size="md"
              variant="primary"
            >
              <Plus size={16} />
              {t("settings.leave.type_manager.get_started", "Get Started")}
            </Button>
          }
          description={t("settings.leave.type_manager.empty_desc", "Your organization is fresh and clean. Start by creating leave types manually or use our professional templates.")}
          icon={<Sparkles className="text-primary-400" size={40} />}
          title={t("settings.leave.type_manager.empty_title", "No Leave Types Configured")}
        />
      )}

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

// EditLeaveTypeModal extracted to ./leave-policy-components/edit-leave-type-modal.tsx
