import type { TFunction } from "i18next";

import type { OrganizationMember } from "@/lib/api";
import type { WorkLocation } from "@/lib/api/locations";

import { Typography } from "@/components/ui/typography";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import { Employee, Employment, Employer } from "@/lib/api";

interface Props {
  activeEmployment?: Employment;
  editingEmployee: Employee | null;
  employers: Employer[];
  locations: WorkLocation[];
  locationsUnavailable?: boolean;
  members: OrganizationMember[];
  prefillEnrollmentId: string;
  selectedEmployerId: string;
  selectedLocationId: string;
  selectedUserId: string;
  setSelectedEmployerId: (val: string) => void;
  setSelectedLocationId: (val: string) => void;
  setSelectedUserId: (val: string) => void;
  t: TFunction;
}

export default function EmployeeFormInputs({
  editingEmployee,
  activeEmployment,
  prefillEnrollmentId,
  members,
  selectedUserId,
  setSelectedUserId,
  locations,
  locationsUnavailable = false,
  selectedLocationId,
  setSelectedLocationId,
  employers,
  selectedEmployerId,
  setSelectedEmployerId,
  t,
}: Props) {
  return (
    <>
      <FormField
        data-testid="employees-form-name-input"
        defaultValue={editingEmployee?.full_name}
        label={t("employees.form.full_name")}
        name="full_name"
        required
      />
      <FormField
        data-testid="employees-form-id-input"
        defaultValue={editingEmployee?.employee_id_number ?? prefillEnrollmentId}
        inputClassName="font-mono tracking-widest"
        label={t("employees.form.employee_id")}
        name="employee_id_number"
        required
      />
      <FormField
        data-testid="employees-form-hire-date-input"
        defaultValue={activeEmployment?.start_date}
        label={t("employees.form.hire_date")}
        name="hire_date"
        required
        type="date"
      />
      <div className="space-y-3 px-1">
        <label className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
          {t("employees.form.link_user")}
        </label>
        <Combobox
          clearOnEscape
          data-testid="employees-form-user-combobox"
          onChange={setSelectedUserId}
          options={members.map((m) => ({
            value: m.user_id,
            label: m.full_name ?? m.email,
            description: m.email,
          }))}
          placeholder={t("employees.form.select_user")}
          value={selectedUserId}
        />
      </div>
      <div className="space-y-3 px-1">
        <div aria-labelledby="employees-form-location-label" role="group">
          <span className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]" id="employees-form-location-label">
            {t("employees.form.location")}
          </span>
          {locationsUnavailable ? (
          <Typography variant="p" className="text-sm text-slate-500 dark:text-slate-400" role="status">
            {t("employees.form.location_unavailable")}
          </Typography>
        ) : (
          <Combobox
            clearOnEscape
            data-testid="employees-form-location-combobox"
            id="employees-form-location-combobox"
            onChange={setSelectedLocationId}
            options={locations.map((loc) => ({
              value: loc.id,
              label: loc.name,
              description: loc.is_headquarters ? "Headquarters" : loc.timezone,
            }))}
            placeholder="Chọn địa điểm làm việc"
            value={selectedLocationId}
          />
        )}
        </div>
      </div>
      <div className="space-y-3 px-1">
        <label className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]" htmlFor="employees-form-employer-combobox">
          Pháp nhân trực thuộc
        </label>
        <Combobox
          clearOnEscape
          data-testid="employees-form-employer-combobox"
          id="employees-form-employer-combobox"
          onChange={setSelectedEmployerId}
          options={employers.map((emp) => ({
            value: emp.id,
            label: emp.name,
            description: emp.tax_code ? `MST: ${emp.tax_code}` : undefined,
          }))}
          placeholder="Chọn pháp nhân trực thuộc"
          value={selectedEmployerId}
        />
      </div>
      <div className="flex items-center gap-4 py-2 px-2 group cursor-pointer">
        <input
          className="w-7 h-7 text-primary-600 dark:text-primary-400 rounded-xl border-slate-200 dark:border-slate-700 focus:ring-primary-500 transition-all cursor-pointer shadow-inner"
          data-testid="employees-form-active-checkbox"
          defaultChecked={editingEmployee ? !!editingEmployee.active_employment_id : true}
          id="is_active"
          name="is_active"
          type="checkbox"
        />
        <label
          className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest cursor-pointer group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors"
          htmlFor="is_active"
        >
          {t("employees.form.is_active")}
        </label>
      </div>
    </>
  );
}
