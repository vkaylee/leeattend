import React, { useState, useEffect } from "react";

import { Shield, ShieldAlert, ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/Button";
import Modal from "@/components/ui/Modal";

import { useWorkspaceRoles } from "@/hooks/use-workspace";
import { cn } from "@/lib/utils";

interface RoleFormModalProps {
  isOpen: boolean;
  onClose: () => void;
  editingRole?: { id: string; name: string; description: string; color: string } | null;
}

const AVAILABLE_COLORS = [
  { id: "slate", label: "Slate", class: "bg-slate-500" },
  { id: "rose", label: "Rose", class: "bg-rose-500" },
  { id: "indigo", label: "Indigo", class: "bg-indigo-500" },
  { id: "emerald", label: "Emerald", class: "bg-emerald-500" },
  { id: "amber", label: "Amber", class: "bg-amber-500" },
  { id: "cyan", label: "Cyan", class: "bg-cyan-500" },
];

export default function RoleFormModal({ isOpen, onClose, editingRole }: RoleFormModalProps) {
  const { t } = useTranslation();
  const { createRole, updateRole, isCreating, isUpdating } = useWorkspaceRoles();
  
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [color, setColor] = useState("slate");

  useEffect(() => {
    if (editingRole && isOpen) {
      // eslint-disable-next-line
      setName(editingRole.name);
      setDescription(editingRole.description || "");
      setColor(editingRole.color || "slate");
    } else if (isOpen) {
      setName("");
      setDescription("");
      setColor("slate");
    }
  }, [editingRole, isOpen]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      if (editingRole) {
        await updateRole({ roleId: editingRole.id, data: { name, description, color } });
      } else {
        await createRole({ name, description, color });
      }
      onClose();
    } catch (err) {
      console.error("Failed to save role", err);
    }
  };

  const isSaving = isCreating || isUpdating;

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={editingRole ? "Edit Custom Role" : "Create Custom Role"}>
      <form onSubmit={handleSubmit} className="space-y-6 pt-4">
        <div className="space-y-2">
          <label className="text-sm font-bold text-slate-700 dark:text-slate-300">Role Name</label>
          <input
            required
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. Finance Manager"
            className="w-full bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 outline-none transition-all"
          />
        </div>

        <div className="space-y-2">
          <label className="text-sm font-bold text-slate-700 dark:text-slate-300">Description</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="What can this role do?"
            rows={3}
            className="w-full bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 outline-none transition-all resize-none"
          />
        </div>

        <div className="space-y-3">
          <label className="text-sm font-bold text-slate-700 dark:text-slate-300">Role Color</label>
          <div className="flex flex-wrap gap-3">
            {AVAILABLE_COLORS.map(c => (
              <button
                key={c.id}
                type="button"
                onClick={() => setColor(c.id)}
                className={cn(
                  "w-8 h-8 rounded-full flex items-center justify-center transition-all",
                  c.class,
                  color === c.id ? "ring-4 ring-offset-2 ring-primary-500/30 dark:ring-offset-slate-900 scale-110" : "hover:scale-110"
                )}
                title={c.label}
              />
            ))}
          </div>
        </div>

        <div className="flex items-center justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={isSaving}>
            {editingRole ? "Save Changes" : "Create Role"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
