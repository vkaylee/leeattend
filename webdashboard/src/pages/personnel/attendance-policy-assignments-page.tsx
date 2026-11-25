import { useNavigate, useSearch } from "@tanstack/react-router";
import { ShieldCheck } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { PolicyAssignmentsSection } from "@/components/features/attendance/policy-assignments-section";
import Button from "@/components/ui/button";
import { CONTROL_FOCUS_RING } from "@/components/ui/control-styles";
import { PageContainer } from "@/components/ui/page-container";
import { PageHeader } from "@/components/ui/page-header";
import { SearchInput } from "@/components/ui/search-input";

export default function AttendancePolicyAssignmentsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const search = useSearch({ strict: false });
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
        <SearchInput
          aria-label={t("common.search", { defaultValue: "Search" })}
          containerClassName="min-w-0 flex-1"
          onChange={(event) => { setQuery(event.target.value); }}
          onClear={() => { setQuery(""); updateSearch({ q: undefined, page: 1 }); }}
          onKeyDown={(event) => { if (event.key === "Enter") updateSearch({ q: query || undefined, page: 1 }); }}
          placeholder={t("common.search", { defaultValue: "Search" })}
          size="sm"
          value={query}
        />
        <select
          aria-label={t("common.target_type", { defaultValue: "Target type" })}
          className={`rounded-xl border border-slate-200 bg-transparent px-3 text-sm dark:border-slate-700 ${CONTROL_FOCUS_RING}`}
          onChange={(event) => { updateSearch({ target_type: event.target.value || undefined, page: 1 }); }}
          value={search.target_type ?? ""}
        >
          <option value="">{t("common.all", { defaultValue: "All" })}</option>
          <option value="GROUP">{t("attendance_policy_overrides.labels.department", { defaultValue: "Department" })}</option>
          <option value="USER">{t("attendance_policy_overrides.labels.employee", { defaultValue: "Employee" })}</option>
        </select>
        <Button onClick={() => { updateSearch({ q: query || undefined, page: 1 }); }} variant="outline">
          {t("common.search", { defaultValue: "Search" })}
        </Button>
      </div>
      <PolicyAssignmentsSection options={{ q: search.q }} />
    </PageContainer>
  );
}
