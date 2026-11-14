import { useQuery } from "@tanstack/react-query";
import { Tags, Plus, Pencil, Trash2, ShieldAlert, Clock, Heart, Baby, Ban } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { SettingsHeader } from "@/components/ui/settings/settings-header";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import {
  fetchConditionTypes,
  type ConditionTypeDefinition,
  type CreateConditionTypeRequest,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";
import { cn } from "@/lib/utils";

import { ConditionTypeForm } from "./components/condition-type-form";
import { useConditionTypeActions } from "./hooks/use-condition-type-actions";


const LUCIDE_ICONS: Record<string, React.ReactNode> = {
  Clock: <Clock size={18} />,
  Baby: <Baby size={18} />,
  Heart: <Heart size={18} />,
  ShieldAlert: <ShieldAlert size={18} />,
  Ban: <Ban size={18} />,
};

const CATEGORY_LABELS: Record<string, string> = {
  working_hours: "Working Hours",
  leave: "Leave",
  general: "General",
};

function ConditionTypeRow({ item, onDelete, onEdit }: {
  item: ConditionTypeDefinition;
  onDelete: (item: ConditionTypeDefinition) => void;
  onEdit: () => void;
}) {
  return (
    <div className="flex items-start justify-between">
      <div className="flex gap-5">
        <div className="w-12 h-12 rounded-2xl bg-slate-50 dark:bg-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 shadow-inner">
          {LUCIDE_ICONS[item.icon ?? ""] ?? <Clock size={18} />}
        </div>
        <div>
          <div className="flex items-center gap-3 mb-1">
            <Typography className="font-bold text-slate-700 dark:text-slate-200 text-base tracking-tight" variant="h4">
              {item.name_i18n.en || item.code}
            </Typography>
            <span className="text-[9px] font-bold uppercase tracking-widest text-slate-400 bg-slate-100 dark:bg-slate-800 px-2 py-0.5 rounded-full">{item.code}</span>
            {item.is_system ? <span className="text-[9px] font-bold uppercase tracking-widest text-primary-600 bg-primary-50 dark:bg-primary-950 px-2 py-0.5 rounded-full">SYSTEM</span> : null}
            <span className={cn(
              "px-2.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-widest border",
              item.default_is_paid
                ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
                : "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800",
            )}>{item.default_is_paid ? "PAID" : "UNPAID"}</span>
          </div>
          <div className="flex items-center gap-3 text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest mt-1">
            <span>{CATEGORY_LABELS[item.category] ?? item.category}</span>
            <span>·</span>
            <span>{item.default_minute_allowance}m default</span>
          </div>
        </div>
      </div>
      <div className="flex gap-2 opacity-0 group-hover:opacity-100 transition-all -translate-y-2 group-hover:translate-y-0">
        <Button className="p-3 rounded-2xl bg-slate-50 dark:bg-slate-800 text-slate-500 hover:bg-primary-50 hover:text-primary-600 transition-all shadow-sm" onClick={onEdit} variant="ghost">
          <Pencil size={16} />
        </Button>
        {!item.is_system && (
          <Button className="p-3 rounded-2xl bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 hover:bg-rose-600 hover:text-white transition-all shadow-sm" onClick={() => { onDelete(item); }} variant="ghost">
            <Trash2 size={16} />
          </Button>
        )}
      </div>
    </div>
  );
}

export default function ConditionTypesPage() {
  const { t } = useTranslation();
  const { confirm, ConfirmationDialog } = useConfirm();

  const [isAdding, setIsAdding] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);

  const { data: types = [], isLoading } = useQuery({
    queryKey: createQueryKey("condition-types"),
    queryFn: () => fetchConditionTypes(),
  });

  const { createMutation, updateMutation, deleteMutation } = useConditionTypeActions();

  const handleDelete = async (item: ConditionTypeDefinition) => {
    const ok = await confirm({
      title: t("common.delete_confirm"),
      message: t("settings.condition_types.confirm_delete", {
        defaultValue: "Delete this condition type? Existing conditions using it will not be affected.",
      }),
      variant: "danger",
    });
    if (ok) deleteMutation.mutate(item.id);
  };

  if (isLoading) {
    return (
      <div className="space-y-4" data-testid="condition-types-skeleton">
        <Skeleton className="h-8 w-48" />
        {[1, 2, 3].map((i) => (
          <Skeleton className="h-24 w-full rounded-2xl" key={i} />
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <SettingsHeader
        actions={
          !isAdding ? (
            <Button
              className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal"
              icon={<Plus size={16} />}
              onClick={() => { setIsAdding(true); setEditingId(null); }}
              variant="primary"
            >
              {t("common.add_new", "Thêm mới")}
            </Button>
          ) : undefined
        }
        description={t("settings.condition_types.subtitle", { defaultValue: "Cấu hình các loại điều kiện làm việc áp dụng cho nhân viên" })}
        icon={<Tags size={20} />}
        title={t("settings.categories.condition_types", { defaultValue: "Điều kiện làm việc" })}
      />

      {/* Add Form */}
      {isAdding ? (
        <ConditionTypeForm
          isSubmitting={createMutation.isPending}
          onCancel={() => {
            setIsAdding(false);
            createMutation.reset();
          }}
          onSave={(data) => {
            createMutation.mutate(data as CreateConditionTypeRequest, {
              onSuccess: () => { setIsAdding(false); },
            });
          }}
          t={(k) => t(k)}
        />
      ) : null}

      {/* Type Cards */}
      <div className="grid grid-cols-1 gap-4">
        {types.length === 0 ? (
          <EmptyState
            action={
              <Button
                className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal"
                icon={<Plus size={16} />}
                onClick={() => { setIsAdding(true); setEditingId(null); }}
                variant="primary"
              >
                {t("common.add_new", "Thêm mới")}
              </Button>
            }
            description={t("settings.condition_types.no_types_desc", { defaultValue: "Thêm điều kiện làm việc đầu tiên để thiết lập chế độ làm việc cho nhân sự." })}
            icon={<Tags size={40} />}
            title={t("settings.condition_types.no_types", { defaultValue: "Chưa có loại điều kiện làm việc" })}
          />
        ) : (
          types.map((item) => (
            <div
              className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] p-6 hover:shadow-premium-xl transition-all group"
              key={item.id}
            >
              {editingId === item.id ? (
                <ConditionTypeForm
                  initial={item}
                  isSubmitting={updateMutation.isPending}
                  onCancel={() => { setEditingId(null); updateMutation.reset(); }}
                  onSave={(data) => { updateMutation.mutate({ id: item.id, req: data }, { onSuccess: () => { setEditingId(null); } }); }}
                  t={(k) => t(k)}
                />
              ) : (
                <ConditionTypeRow
                  item={item}
                  onDelete={handleDelete}
                  onEdit={() => { setEditingId(item.id); setIsAdding(false); }}
                />
              )}
            </div>
          ))
        )}
      </div>

      <ConfirmationDialog />
    </div>
  );
}
