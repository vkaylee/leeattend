import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import {
  fetchSmtpConfig,
  saveSmtpConfig,
  testSmtp,
  type HttpApiConfig,
  type SmtpConfigData,
} from "@/lib/api/smtp-config";

const DEFAULT_HTTP_API_TEMPLATE = JSON.stringify(
  {
    from: { email: "{{from_email}}", name: "{{from_name}}" },
    to: [{ email: "{{to}}" }],
    subject: "{{subject}}",
    text: "{{body}}",
  },
  null,
  2,
);

interface TestResult {
  message: string;
  success: boolean;
}

function buildForm(cfg: SmtpConfigData | null | undefined, ov: Partial<SmtpConfigData>): SmtpConfigData {
  return {
    host: ov.host ?? cfg?.host ?? "",
    port: ov.port ?? cfg?.port ?? 587,
    username: ov.username ?? cfg?.username ?? "",
    password: ov.password ?? cfg?.password ?? "",
    from: ov.from ?? cfg?.from ?? "",
    base_url: ov.base_url ?? cfg?.base_url ?? "",
    http_api: cfg?.http_api ?? undefined,
  };
}

function buildHttpForm(
  cfg: SmtpConfigData | null | undefined,
  ov: Partial<SmtpConfigData>,
  hov: Partial<HttpApiConfig>,
): HttpApiConfig {
  const src = ov.http_api ?? cfg?.http_api;
  return {
    url: hov.url ?? src?.url ?? "",
    auth_header: hov.auth_header ?? src?.auth_header ?? "",
    auth_value: hov.auth_value ?? src?.auth_value ?? "",
    success_field: hov.success_field ?? src?.success_field ?? "",
    success_value: hov.success_value ?? src?.success_value ?? true,
    body_template: hov.body_template ?? src?.body_template ?? "",
  };
}

function buildSavePayload(smtp: SmtpConfigData, http: HttpApiConfig): SmtpConfigData {
  const payload: SmtpConfigData = {
    host: smtp.host || undefined,
    port: smtp.port || undefined,
    username: smtp.username || undefined,
    password: smtp.password || undefined,
    from: smtp.from || undefined,
    base_url: smtp.base_url || undefined,
  };
  if (http.url.trim()) {
    payload.http_api = {
      url: http.url.trim(),
      auth_header: http.auth_header || undefined,
      auth_value: http.auth_value || undefined,
      success_field: http.success_field || undefined,
      success_value: http.success_field ? (http.success_value ?? true) : undefined,
      body_template: http.body_template.trim(),
    };
  } else {
    payload.http_api = null;
  }
  return payload;
}

export function useSmtpConfig() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: config, isLoading } = useQuery({
    queryKey: ["smtp-config"],
    queryFn: fetchSmtpConfig,
  });

  const [dirty, setDirty] = useState(false);
  const [testEmail, setTestEmail] = useState("");
  const [showHttpApi, setShowHttpApi] = useState(() => !!config?.http_api);
  const [showDefaultConfirm, setShowDefaultConfirm] = useState(false);

  const [testResult, setTestResult] = useState<TestResult | null>(null);
  const [testing, setTesting] = useState(false);
  const [verifiedAt, setVerifiedAt] = useState<Date | null>(() =>
    config?.last_verified_at ? new Date(config.last_verified_at) : null,
  );

  const [overrides, setOverrides] = useState<Partial<SmtpConfigData>>({});
  const [httpOverrides, setHttpOverrides] = useState<Partial<HttpApiConfig>>({});

  const form: SmtpConfigData = useMemo(() => buildForm(config, overrides), [config, overrides]);
  const httpForm: HttpApiConfig = useMemo(
    () => buildHttpForm(config, overrides, httpOverrides),
    [config, overrides, httpOverrides],
  );

  const handleFillDefault = useCallback(() => {
    setShowDefaultConfirm(false);
    setHttpOverrides({
      url: "https://send.api.mailtrap.io/api/send",
      auth_header: "Api-Token",
      auth_value: "{{password}}",
      success_field: "success",
      success_value: true,
      body_template: DEFAULT_HTTP_API_TEMPLATE,
    });
    setDirty(true);
  }, []);

  const updateField = (field: keyof SmtpConfigData, value: string | number) => {
    setOverrides((prev) => ({ ...prev, [field]: value }));
    setDirty(true);
  };

  const saveMutation = useMutation({
    mutationFn: saveSmtpConfig,
    onSuccess: () => {
      toast.success(t("common.saved_success"));
      void queryClient.invalidateQueries({ queryKey: ["smtp-config"] });
      setDirty(false);
      setVerifiedAt(null);
      setTestResult(null);
    },
    onError: (err: Error) => {
      toast.error(err.message);
    },
  });

  const handleSave = () => {
    const payload = buildSavePayload(form, httpForm);
    saveMutation.mutate(payload);
  };

  const handleTest = async () => {
    if (!testEmail.trim()) return;
    setTesting(true);
    setTestResult(null);
    try {
      const result = await testSmtp(testEmail.trim());
      setTestResult(result);
      if (result.success) {
        setVerifiedAt(new Date());
      }
    } catch (err: unknown) {
      setTestResult({
        success: false,
        message: err instanceof Error ? err.message : "Test failed",
      });
    } finally {
      setTesting(false);
    }
  };

  const verifiedTime = useMemo(() => {
    if (!verifiedAt) return null;
    return verifiedAt.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
  }, [verifiedAt]);

  return {
    config,
    isLoading,
    dirty,
    markDirty: () => { setDirty(true); },
    testEmail,
    setTestEmail,
    showHttpApi,
    setShowHttpApi,
    showDefaultConfirm,
    setShowDefaultConfirm,
    testResult,
    setTestResult,
    testing,
    verifiedAt,
    verifiedTime,
    form,
    httpForm,
    httpOverrides,
    setHttpOverrides,
    handleFillDefault,
    updateField,
    saveMutation,
    handleSave,
    handleTest,
    t,
  };
}
