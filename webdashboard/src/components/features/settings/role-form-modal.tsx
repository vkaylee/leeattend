import React, { useState, useEffect } from "react";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useWorkspaceRoles } from "@/hooks/use-workspace";
import { cn } from "@/lib/utils";

interface RoleFormModalProps {
  editingRole?: { id: string; name: string; description: string; color: string } | null;
  isOpen: boolean;
  onClose: () => void;
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
  const { createRole, updateRole, isCreating, isUpdating } = useWorkspaceRoles();
  
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [color, setColor] = useState("slate");

  useEffect(() => {
    if (editingRole && isOpen) {
      // eslint-disable-next-line -- Derived state synced from props; intentional pattern
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
    } catch (error) {
      console.error("Failed to save role", error);
    }
  };

  const isSaving = isCreating || isUpdating;

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={editingRole ? "Edit Custom Role" : "Create Custom Role"}>
      <form className="space-y-6 pt-4" onSubmit={handleSubmit}>
        <div className="space-y-2">
          <label className="text-sm font-bold text-slate-700 dark:text-slate-300" htmlFor="role-name">Role Name</label>
          <input
            className="w-full bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 outline-none transition-all"
            id="role-name"
 
            onChange={(e) => { setName(e.target.value); }}
            placeholder="e.g. Finance Manager"
            required
            value={name}
          />
        </div>

        <div className="space-y-2">
          <label className="text-sm font-bold text-slate-700 dark:text-slate-300" htmlFor="role-desc">Description</label>
          <textarea
            className="w-full bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 outline-none transition-all resize-none"
 
            id="role-desc"
            onChange={(e) => { setDescription(e.target.value); }}
            placeholder="What can this role do?"
            rows={3}
            value={description}
          />
        </div>

        <div className="space-y-3">
          <div className="text-sm font-bold text-slate-700 dark:text-slate-300" id="role-color-label">Role Color</div>
          <div aria-labelledby="role-color-label" className="flex flex-wrap gap-3" role="radiogroup">
            { }
            {AVAILABLE_COLORS.map(c => (
              <button
                className={cn(
                  "w-8 h-8 rounded-full flex items-center justify-center transition-all",
                  c.class,
                  color === c.id ? "ring-4 ring-offset-2 ring-primary-500/30 dark:ring-offset-slate-900 scale-110" : "hover:scale-110"
                )}
                key={c.id}
                onClick={() => { setColor(c.id); }}
                title={c.label}
                type="button"
              />
            ))}
          </div>
        </div>

        <div className="flex items-center justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
          <Button onClick={onClose} type="button" variant="ghost">
            Cancel
          </Button>
          <Button loading={isSaving} type="submit">
            {editingRole ? "Save Changes" : "Create Role"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
