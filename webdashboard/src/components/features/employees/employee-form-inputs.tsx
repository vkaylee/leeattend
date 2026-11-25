import type { TFunction } from "i18next";

import type { Employee, Employment, Employer } from "@/lib/api";
import type { WorkLocation } from "@/lib/api/locations";

import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";

interface Props {
  activeEmployment?: Employment;
  basicOnly?: boolean;
  dateOfBirth?: string;
  editingEmployee: Employee | null;
  employeeCode?: string;
  employerError?: string;
  employers: Employer[];
  employmentOnly?: boolean;
  fullName?: string;
  hireDate?: string;
  identityDisabled?: boolean;
  locations: WorkLocation[];
  locationsUnavailable?: boolean;
  nationalId?: string;
  onDateOfBirthChange?: (val: string) => void;
  onEmployeeCodeChange?: (val: string) => void;
  onFullNameChange?: (val: string) => void;
  onHireDateChange?: (val: string) => void;
  onNationalIdChange?: (val: string) => void;
  onPhoneChange?: (val: string) => void;
  phone?: string;
  prefillEnrollmentId: string;
  reuseMode?: boolean;
  selectedEmployerId: string;
  selectedLocationId: string;
  setSelectedEmployerId: (val: string) => void;
  setSelectedLocationId: (val: string) => void;
  showIdentityFields?: boolean;
  t: TFunction;
}

function IdentityInputs({
  dateOfBirth,
  disabled,
  nationalId,
  onDateOfBirthChange,
  onNationalIdChange,
  onPhoneChange,
  phone,
  t,
}: {
  dateOfBirth?: string;
  disabled: boolean;
  nationalId?: string;
  onDateOfBirthChange?: (val: string) => void;
  onNationalIdChange?: (val: string) => void;
  onPhoneChange?: (val: string) => void;
  phone?: string;
  t: TFunction;
}) {
  return (
    <div className="space-y-4 border-t border-slate-100 pt-4 dark:border-slate-800" data-testid="employees-form-identity-fields">
      <FormField
        data-testid="employees-form-national-id-input"
        disabled={disabled}
        label={t("employees.form.national_id", { defaultValue: "CCCD / CMND" })}
        name="national_id"
        onChange={onNationalIdChange ? (e) => { onNationalIdChange(e.target.value); } : undefined}
        placeholder={t("employees.form.national_id_placeholder", { defaultValue: "VD: 001234567890" })}
        value={nationalId ?? ""}
      />
      <FormField
        data-testid="employees-form-phone-input"
        disabled={disabled}
        label={t("employees.form.phone", { defaultValue: "Số điện thoại (E.164)" })}
        name="phone"
        onChange={onPhoneChange ? (e) => { onPhoneChange(e.target.value); } : undefined}
        placeholder={t("employees.form.phone_placeholder", { defaultValue: "+84912345678" })}
        type="tel"
        value={phone ?? ""}
      />
      <FormField
        data-testid="employees-form-dob-input"
        disabled={disabled}
        label={t("employees.form.date_of_birth", { defaultValue: "Ngày sinh" })}
        name="date_of_birth"
        onChange={onDateOfBirthChange ? (e) => { onDateOfBirthChange(e.target.value); } : undefined}
        type="date"
        value={dateOfBirth ?? ""}
      />
    </div>
  );
}

function BasicFields(props: Props) {
  const { activeEmployment, dateOfBirth, editingEmployee, employeeCode, fullName, hireDate, identityDisabled = false, nationalId, onDateOfBirthChange, onEmployeeCodeChange, onFullNameChange, onHireDateChange, onNationalIdChange, onPhoneChange, phone, prefillEnrollmentId, reuseMode = false, showIdentityFields = false, t } = props;
  const fallbackCode = editingEmployee?.employee_id_number ?? prefillEnrollmentId;

  return (
    <>
      <FormField
        data-testid="employees-form-name-input"
        defaultValue={onFullNameChange ? undefined : editingEmployee?.full_name}
        disabled={reuseMode}
        label={t("employees.form.full_name")}
        name="full_name"
        onChange={onFullNameChange ? (e) => { onFullNameChange(e.target.value); } : undefined}
        required={!reuseMode}
        value={onFullNameChange ? (fullName ?? "") : undefined}
      />
      <FormField
        data-testid="employees-form-id-input"
        defaultValue={onEmployeeCodeChange ? undefined : fallbackCode}
        disabled={reuseMode}
        inputClassName="font-mono tracking-widest"
        label={t("employees.form.employee_id")}
        name="employee_id_number"
        onChange={onEmployeeCodeChange ? (e) => { onEmployeeCodeChange(e.target.value); } : undefined}
        required={!reuseMode}
        value={onEmployeeCodeChange ? (employeeCode ?? "") : undefined}
      />
      <FormField
        data-testid="employees-form-hire-date-input"
        defaultValue={onHireDateChange ? undefined : activeEmployment?.start_date}
        label={t("employees.form.hire_date")}
        name="hire_date"
        onChange={onHireDateChange ? (e) => { onHireDateChange(e.target.value); } : undefined}
        required
        type="date"
        value={onHireDateChange ? (hireDate ?? "") : undefined}
      />
      {showIdentityFields ? (
        <IdentityInputs
          dateOfBirth={dateOfBirth}
          disabled={identityDisabled || reuseMode}
          nationalId={nationalId}
          onDateOfBirthChange={onDateOfBirthChange}
          onNationalIdChange={onNationalIdChange}
          onPhoneChange={onPhoneChange}
          phone={phone}
          t={t}
        />
      ) : null}
    </>
  );
}

function EmploymentFields({
  editingEmployee,
  employerError,
  employers,
  locations,
  locationsUnavailable = false,
  selectedEmployerId,
  selectedLocationId,
  setSelectedEmployerId,
  setSelectedLocationId,
  t,
}: Props) {
  return (
    <>
      <div className="space-y-3 px-1">
        <div aria-labelledby="employees-form-location-label" role="group">
          <span className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]" id="employees-form-location-label">
            {t("employees.form.location")}
          </span>
          {locationsUnavailable ? (
            <Typography className="text-sm text-slate-500 dark:text-slate-400" role="status" variant="p">
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
              placeholder={t("employees.form.select_location", { defaultValue: "Chọn địa điểm làm việc" })}
              value={selectedLocationId}
            />
          )}
        </div>
      </div>
      <div className="space-y-3 px-1">
        <label className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]" htmlFor="employees-form-employer-combobox">
          {t("employees.form.employer", { defaultValue: "Pháp nhân trực thuộc" })}
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
          placeholder={t("employees.form.select_employer", { defaultValue: "Chọn pháp nhân trực thuộc" })}
          value={selectedEmployerId}
        />
        {employerError ? (
          <Typography className="text-xs font-medium text-rose-500" data-testid="employees-form-employer-error" role="alert" variant="p">
            {employerError}
          </Typography>
        ) : null}
      </div>
      <div className="flex items-center gap-4 py-2 px-2 group cursor-pointer">
        <input
          className="w-7 h-7 text-primary-600 dark:text-primary-400 rounded-xl border-slate-200 dark:border-slate-700 focus:ring-primary-500 transition-all cursor-pointer shadow-inner"
          data-testid="employees-form-active-checkbox"
          defaultChecked={editingEmployee ? Boolean(editingEmployee.active_employment_id) : true}
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

export default function EmployeeFormInputs(props: Props) {
  const { basicOnly = false, employmentOnly = false } = props;
  return (
    <>
      {employmentOnly ? null : <BasicFields {...props} />}
      {basicOnly ? null : <EmploymentFields {...props} />}
    </>
  );
}
