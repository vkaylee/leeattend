import { Wallet, AlertTriangle, Pencil, Plus, Trash2, Sparkles } from "lucide-react";
import React, { useState } from "react";

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

export function LeaveTypeManager({ wsId }: LeaveTypeManagerProps) {
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
      <div className="py-12 text-center text-slate-400 dark:text-slate-500 animate-pulse">
        Loading leave types...
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center px-1">
        <Typography className="text-xs font-medium text-slate-500 dark:text-slate-400 max-w-md italic" variant="p">
          Define the categories of leave available in your organization (e.g., Annual Leave, Sick Leave).
        </Typography>
        <button
          className="flex items-center gap-2 rounded-xl bg-primary-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100"
          onClick={() => {
            setIsAddingType(true);
          }}
        >
          <Plus size={14} /> Add Leave Type
        </button>
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
                    <button
                      className="p-2 text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:text-primary-400 bg-slate-50 dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-800"
                      onClick={() => {
                        setEditingType(type);
                      }}
                    >
                      <Pencil size={16} />
                    </button>
                    <button
                      className="p-2 text-slate-400 dark:text-slate-500 hover:text-rose-600 dark:text-rose-300 bg-slate-50 dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-800"
                      onClick={() => {
                        setDeletingType(type);
                      }}
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </div>
                <Typography className="mt-5 text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight leading-none uppercase" variant="h3">
                  {type.name}
                </Typography>
                <Typography className="mt-1 text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
                  Code: {type.code}
                </Typography>
                <div className="mt-4 flex gap-2">
                  <span
                    className={cn(
                      "px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border",
                      type.is_paid
                        ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
                        : "bg-amber-50 dark:bg-amber-950 text-amber-700 dark:text-amber-300 border-amber-100 dark:border-amber-800"
                    )}
                  >
                    {type.is_paid ? "Paid" : "Unpaid"}
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <EmptyState
          action={
            <button
              className="flex items-center gap-2 rounded-xl bg-primary-600 px-8 py-3 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-200 dark:shadow-none"
              onClick={() => {
                setIsAddingType(true);
              }}
            >
              <Plus size={16} />
              Get Started
            </button>
          }
          description="Your workspace is fresh and clean. Start by creating leave types manually or use our professional templates."
          icon={<Sparkles className="text-primary-400" size={40} />}
          title="No Leave Types Configured"
        />
      )}

      {isAddingType ? (
        <AddLeaveTypeModal
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
          title="Confirm Delete"
        >
          <div className="p-8">
            <Typography className="text-sm" variant="p">Are you sure you want to delete &quot;{deletingType.name}&quot;?</Typography>
            <div className="mt-8 flex justify-end gap-3">
              <button
                className="px-6 py-2 text-xs font-bold text-slate-400 dark:text-slate-500"
                onClick={() => {
                  setDeletingType(null);
                }}
              >
                Cancel
              </button>
              <button
                className="rounded-xl bg-rose-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase disabled:opacity-50"
                disabled={isDeleting}
                onClick={() => {
                  triggerDelete(deletingType.id);
                }}
              >
                Delete
              </button>
            </div>
          </div>
        </Modal>
      ) : null}
    </div>
  );
}

interface AddLeaveTypeModalProps {
  isSaving: boolean;
  onClose: () => void;
  onSave: (data: { code: string; name: string; is_paid: boolean }) => void;
}

function AddLeaveTypeModal({ onClose, onSave, isSaving }: AddLeaveTypeModalProps) {
  const [step, setStep] = useState<"template" | "form">("template");
  const [formData, setFormData] = useState({ code: "", name: "", is_paid: true });

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
          ? "Start faster with a professional setup"
          : `Refining ${formData.name || "New Leave Type"}`
      }
      title={step === "template" ? "Choose a Template" : "Customize Leave Type"}
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
  const [config, setConfig] = useState(
    leaveType.policy_config ?? {
      approval_required: true,
      requires_document: false,
      allow_negative_balance: false,
    }
  );

  return (
    <Modal
      isOpen={true}
      maxWidth="md"
      onClose={onClose}
      subtitle={`Defining rules for ${leaveType.name}`}
      title="Leave Policy Configuration"
    >
      <form
        className="space-y-6"
        onSubmit={(e) => {
          e.preventDefault();
          onSave(config);
        }}
      >
        <div className="rounded-[2rem] bg-slate-50/50 dark:bg-slate-800/30 border border-slate-100 dark:border-slate-800 overflow-hidden">
          <div className="flex items-center justify-between p-4 border-b border-slate-100 dark:border-slate-800">
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-xl bg-primary-50 dark:bg-primary-950/50 text-primary-600 flex items-center justify-center shadow-inner shrink-0">
                <Plus size={16} />
              </div>
              <div>
                <label className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block" htmlFor="approval-req-input">
                  Approval Required
                </label>
                <Typography className="text-[10px] text-slate-400 dark:text-slate-500" variant="p">Managers must approve requests</Typography>
              </div>
            </div>
            <div className="relative inline-flex items-center cursor-pointer">
              <input
                checked={config.approval_required}
                className="sr-only peer"
                id="approval-req-input"
                onChange={(e) => {
                  setConfig({ ...config, approval_required: e.target.checked });
                }}
                type="checkbox"
              />
              <div className="w-10 h-5.5 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4.5 after:w-4.5 after:transition-all peer-checked:bg-primary-600"></div>
            </div>
          </div>

          <div className="flex items-center justify-between p-4 border-b border-slate-100 dark:border-slate-800">
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-xl bg-blue-50 dark:bg-blue-950/50 text-blue-600 flex items-center justify-center shadow-inner shrink-0">
                <Sparkles size={16} />
              </div>
              <div>
                <label className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block" htmlFor="doc-req-input">
                  Requires Document
                </label>
                <Typography className="text-[10px] text-slate-400 dark:text-slate-500" variant="p">Proof/Medical cert is mandatory</Typography>
              </div>
            </div>
            <div className="relative inline-flex items-center cursor-pointer">
              <input
                checked={config.requires_document}
                className="sr-only peer"
                id="doc-req-input"
                onChange={(e) => {
                  setConfig({ ...config, requires_document: e.target.checked });
                }}
                type="checkbox"
              />
              <div className="w-10 h-5.5 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4.5 after:w-4.5 after:transition-all peer-checked:bg-primary-600"></div>
            </div>
          </div>

          <div className="flex items-center justify-between p-4">
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-xl bg-amber-50 dark:bg-amber-950/50 text-amber-600 flex items-center justify-center shadow-inner shrink-0">
                <AlertTriangle size={16} />
              </div>
              <div>
                <label className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block" htmlFor="negative-bal-input">
                  Negative Balance
                </label>
                <Typography className="text-[10px] text-slate-400 dark:text-slate-500" variant="p">Allow users to take leave beyond 0</Typography>
              </div>
            </div>
            <div className="relative inline-flex items-center cursor-pointer">
              <input
                checked={config.allow_negative_balance}
                className="sr-only peer"
                id="negative-bal-input"
                onChange={(e) => {
                  setConfig({ ...config, allow_negative_balance: e.target.checked });
                }}
                type="checkbox"
              />
              <div className="w-10 h-5.5 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4.5 after:w-4.5 after:transition-all peer-checked:bg-primary-600"></div>
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-3 pt-6 border-t border-slate-50 dark:border-slate-800">
          <button
            className="px-6 py-3 text-[10px] font-bold text-slate-400 dark:text-slate-500 hover:text-slate-600 transition-colors uppercase tracking-widest"
            onClick={onClose}
            type="button"
          >
            Cancel
          </button>
          <button
            className="rounded-2xl bg-primary-600 px-10 py-3.5 text-[10px] font-bold text-white uppercase tracking-[0.2em] shadow-lg shadow-primary-500/20 hover:bg-primary-700 hover:scale-[1.02] active:scale-[0.98] transition-all disabled:opacity-50"
            disabled={isSaving}
            type="submit"
          >
            {isSaving ? "Saving..." : "Save Policy"}
          </button>
        </div>
      </form>
    </Modal>
  );
}
