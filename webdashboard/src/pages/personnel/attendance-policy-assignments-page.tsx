import { useNavigate, useSearch } from "@tanstack/react-router";
import { ShieldCheck } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { PolicyAssignmentsSection } from "@/components/features/attendance/policy-assignments-section";
import Button from "@/components/ui/button";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";

export default function AttendancePolicyAssignmentsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const search = useSearch({ strict: false }) as { q?: string; target_type?: "USER" | "GROUP"; page?: number; limit?: number };
  const [query, setQuery] = useState(search.q ?? "");

  const updateSearch = (patch: Record<string, unknown>) => {
    void navigate({ search: (previous: Record<string, unknown>) => ({ ...previous, ...patch }) as never, replace: true });
  };

  return (
    <PageContainer>
      <PageHeader
        icon={<ShieldCheck size={24} />}
        subtitle={t("common.attendance_policy_assignments_description", { defaultValue: "Manage attendance policy overrides across the organization." })}
        title={t("common.attendance_policy_assignments", { defaultValue: "Attendance Policies" })}
      />
      <div className="flex flex-col gap-3 rounded-2xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-slate-900 sm:flex-row">
        <label className="flex min-w-0 flex-1 items-center gap-2 rounded-xl border border-slate-200 px-3 dark:border-slate-700">
          <span className="sr-only">{t("common.search", { defaultValue: "Search" })}</span>
          <input
            aria-label={t("common.search", { defaultValue: "Search" })}
            className="min-w-0 flex-1 bg-transparent py-2 text-sm outline-none"
            onChange={(event) => { setQuery(event.target.value); }}
            onKeyDown={(event) => { if (event.key === "Enter") updateSearch({ q: query || undefined, page: 1 }); }}
            placeholder={t("common.search", { defaultValue: "Search" })}
            value={query}
          />
        </label>
        <select
          aria-label={t("common.target_type", { defaultValue: "Target type" })}
          className="rounded-xl border border-slate-200 bg-transparent px-3 text-sm dark:border-slate-700"
          onChange={(event) => updateSearch({ target_type: event.target.value || undefined, page: 1 })}
          value={search.target_type ?? ""}
        >
          <option value="">{t("common.all", { defaultValue: "All" })}</option>
          <option value="GROUP">{t("attendance_policy_overrides.labels.department", { defaultValue: "Department" })}</option>
          <option value="USER">{t("attendance_policy_overrides.labels.employee", { defaultValue: "Employee" })}</option>
        </select>
        <Button onClick={() => updateSearch({ q: query || undefined, page: 1 })} variant="outline">
          {t("common.search", { defaultValue: "Search" })}
        </Button>
      </div>
      <PolicyAssignmentsSection
        options={{
          limit: search.limit ?? 50,
          onPageChange: (nextPage) => updateSearch({ page: nextPage }),
          onTargetTypeChange: (targetType) => updateSearch({ target_type: targetType, page: 1 }),
          page: search.page ?? 1,
          q: search.q,
          targetType: search.target_type,
        }}
      />
    </PageContainer>
  );
}
