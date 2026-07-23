import { ChevronDown, Globe, Sparkles } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { HttpApiConfig } from "@/lib/api/smtp-config";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface SmtpHttpApiSectionProps {
  expanded: boolean;
  form: HttpApiConfig;
  onFieldChange: (field: keyof HttpApiConfig, value: string | boolean) => void;
  onFillDefault: () => void;
  onToggle: () => void;
}

export function SmtpHttpApiSection({
  expanded,
  form,
  onToggle,
  onFieldChange,
  onFillDefault,
}: SmtpHttpApiSectionProps) {
  const { t } = useTranslation();

  return (
    <div className="border-t border-slate-100 dark:border-slate-800 pt-5 mt-2">
      <Button
        className="flex items-center gap-2 w-full !text-left font-semibold !h-auto !px-0 !py-0 !rounded-none !tracking-normal !normal-case"
        data-testid="smtp-http-api-toggle"
        onClick={onToggle}
        variant="ghost"
      >
        <Globe className="w-4 h-4" />
        <Typography variant="h3">
          {t("smtp.http_api_section", "HTTP API (Alternative)")}
        </Typography>
        <ChevronDown
          className={`w-4 h-4 ml-auto transition-transform duration-200 ${
            expanded ? "rotate-180" : ""
          }`}
        />
      </Button>
      <Typography className="text-sm text-slate-400 dark:text-slate-500 mt-1 ml-6" variant="p">
        {t("smtp.http_api_desc", "Use a REST API to send emails instead of SMTP. Required on platforms that block outbound SMTP (Railway, Fly.io, etc.). Supports any transactional email provider.")}
      </Typography>

      <div className="flex justify-end mt-2">
        <Button
          onClick={onFillDefault}
          size="sm"
          variant="ghost"
        >
          <Sparkles className="w-3.5 h-3.5 mr-1" />
          {t("smtp.fill_default", "Fill Default Template")}
        </Button>
      </div>

      {expanded ? (
        <div className="ml-6 mt-4 grid gap-4 sm:grid-cols-2 animate-in fade-in slide-in-from-top-2 duration-200">
          <div className="space-y-1.5 sm:col-span-2">
            <label className="text-sm font-medium text-slate-600 dark:text-slate-400">
              {t("smtp.http_api_url", "API URL")} *
            </label>
            <input
              className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
              data-testid="smtp-http-api-url"
              onChange={(e) => { onFieldChange("url", e.target.value); }}
              placeholder="https://send.api.mailtrap.io/api/send"
              type="text"
              value={form.url}
            />
          </div>

          <HttpField
            dataTestId="smtp-http-api-auth-header"
            label={t("smtp.http_api_auth_header", "Auth Header")}
            onChange={(v) => { onFieldChange("auth_header", v); }}
            placeholder="Authorization"
            value={form.auth_header ?? ""}
          />

          <div className="space-y-1.5">
            <HttpLabel label={t("smtp.http_api_auth_value", "Auth Value")} />
            <input
              className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
              data-testid="smtp-http-api-auth-value"
              onChange={(e) => { onFieldChange("auth_value", e.target.value); }}
              placeholder="Bearer {{password}}"
              type="text"
              value={form.auth_value}
            />
            <Typography className="text-[11px] text-slate-400 dark:text-slate-500" variant="p">
              {"{{username}} and {{password}} are replaced with SMTP credentials"}
            </Typography>
          </div>

          <HttpField
            dataTestId="smtp-http-api-success-field"
            label={t("smtp.http_api_success_field", "Success Field")}
            onChange={(v) => { onFieldChange("success_field", v); }}
            placeholder="success"
            value={form.success_field}
          />

          <div className="space-y-1.5">
            <HttpLabel label={t("smtp.http_api_success_value", "Success Value")} />
            <input
              className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
              data-testid="smtp-http-api-success-value"
              onChange={(e) => {
                const raw = e.target.value;
                const parsed = raw === "false" ? false : raw === "true" ? true : raw;
                onFieldChange("success_value", parsed);
              }}
              placeholder="true"
              type="text"
              value={typeof form.success_value === "boolean" ? String(form.success_value) : typeof form.success_value === "string" ? form.success_value : ""}
            />
          </div>

          <div className="space-y-1.5 sm:col-span-2">
            <label className="text-sm font-medium text-slate-600 dark:text-slate-400">
              {t("smtp.http_api_body_template", "JSON Body Template")} *
            </label>
            <textarea
              className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm font-mono placeholder:text-slate-300 dark:placeholder:text-slate-600 resize-y"
              data-testid="smtp-http-api-body-template"
              onChange={(e) => { onFieldChange("body_template", e.target.value); }}
              placeholder={'{"from":{"email":"{{from_email}}","name":"{{from_name}}"},"to":[{"email":"{{to}}"}],"subject":"{{subject}}","text":"{{body}}"}'}
              rows={6}
              value={form.body_template}
            />
            <Typography className="text-[11px] text-slate-400 dark:text-slate-500" variant="p">
              {"Variables: {{from_name}}, {{from_email}}, {{to}}, {{subject}}, {{body}}"}
            </Typography>
          </div>
        </div>
      ) : null}
    </div>
  );
}

/* ── Internal helpers ── */

function HttpLabel({ label }: { label: string }) {
  return <label className="text-sm font-medium text-slate-600 dark:text-slate-400">{label}</label>;
}

function HttpField({
  dataTestId,
  label,
  onChange,
  placeholder,
  value,
}: {
  dataTestId: string;
  label: string;
  onChange: (value: string) => void;
  placeholder: string;
  value: string | undefined;
}) {
  return (
    <div className="space-y-1.5">
      <HttpLabel label={label} />
      <input
        className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
        data-testid={dataTestId}
        onChange={(e) => { onChange(e.target.value); }}
        placeholder={placeholder}
        type="text"
        value={value}
      />
    </div>
  );
}
