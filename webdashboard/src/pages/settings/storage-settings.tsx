import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { toast } from "@/components/ui/toast";
import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";
import {
  useDeleteStorageConfig,
  useStorageConfig,
  useTestStorageConfig,
  useUpsertStorageConfig,
} from "@/hooks/use-storage-settings";
import { logError } from "@/lib/logger";
import { maybeNull } from "@/types/api-overrides";

import { StorageCredentialForm, StorageHeader } from "./storage-form-components";
import { type FormState, EMPTY_FORM } from "./storage-types";

export function StorageSettings() {
  const { t } = useTranslation();
  const { data: config, isLoading } = useStorageConfig();
  const upsertMutation = useUpsertStorageConfig();
  const testMutation = useTestStorageConfig();
  const deleteMutation = useDeleteStorageConfig();

  const [isTesting, setIsTesting] = useState(false);
  const [testResult, setTestResult] = useState<"success" | "error" | null>(null);

  const externalForm = useMemo((): FormState => {
    if (config) {
      return {
        isCustom: true,
        endpointUrl: maybeNull(config.endpoint_url) ?? "",
        region: maybeNull(config.region) ?? "",
        bucketName: maybeNull(config.bucket_name) ?? "",
        accessKey: "",
        secretKey: "",
        usePathStyle: maybeNull(config.use_path_style) ?? true,
      };
    }
    return EMPTY_FORM;
  }, [config]);

  const [form, setForm] = useDerivedStateFrom(externalForm);

  const handleTestConnection = async () => {
    if (!form.endpointUrl || !form.bucketName || !form.accessKey || !form.secretKey || !form.region) {
      toast.error(t("settings.storage.test_missing_fields", "Please fill all fields before testing."));
      return;
    }
    setIsTesting(true);
    setTestResult(null);
    try {
      await testMutation.mutateAsync({
        endpoint_url: form.endpointUrl,
        region: form.region,
        bucket_name: form.bucketName,
        access_key: form.accessKey,
        secret_key: form.secretKey,
        use_path_style: form.usePathStyle,
        max_file_size: 104857600,
        presigned_ttl_secs: 3600,
      });
      setTestResult("success");
      toast.success(t("settings.storage.test_success", "Connection successful!"));
    } catch (e: unknown) {
      logError(e, { component: 'StorageSettings.testConnection' });
      setTestResult("error");
      toast.error(t("settings.storage.test_error", "Connection failed. Please check your credentials."));
    } finally {
      setIsTesting(false);
    }
  };

  const handleSaveConfig = async () => {
    if (form.isCustom) {
      if (!form.endpointUrl || !form.bucketName || !form.accessKey || !form.secretKey || !form.region) {
        toast.error(t("settings.storage.missing_fields", "Please fill in all S3 credentials before saving."));
        return;
      }
      try {
        await upsertMutation.mutateAsync({
          endpoint_url: form.endpointUrl,
          region: form.region,
          bucket_name: form.bucketName,
          access_key: form.accessKey,
          secret_key: form.secretKey,
          use_path_style: form.usePathStyle,
          max_file_size: 104857600,
          presigned_ttl_secs: 3600,
        });
        toast.success(t("settings.storage.save_success", "Storage configuration saved."));
      } catch (e: unknown) {
        logError(e, { component: 'StorageSettings.saveConfig' });
        toast.error(t("settings.storage.save_error", "Failed to save configuration."));
      }
    } else {
      if (config) {
        try {
          await deleteMutation.mutateAsync();
          toast.success(t("settings.storage.revert_success", "Reverted to Central Storage."));
        } catch (e: unknown) {
          logError(e, { component: 'StorageSettings.revertConfig' });
          toast.error(t("settings.storage.revert_error", "Failed to revert configuration."));
        }
      }
    }
  };

  const handleToggleCustom = (enabled: boolean) => {
    setForm(prev => ({ ...prev, isCustom: enabled }));
    setTestResult(null);
  };

  if (isLoading) {
    return <div className="p-8">Loading...</div>;
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-500 max-w-4xl">
      <StorageHeader
        isSaving={upsertMutation.isPending || deleteMutation.isPending}
        onSave={handleSaveConfig}
      />
      <StorageCredentialForm
        form={form}
        hasExistingConfig={!!config}
        isSaving={upsertMutation.isPending}
        isTesting={isTesting}
        onFieldChange={setForm}
        onTestConnection={handleTestConnection}
        onToggleCustom={handleToggleCustom}
        testResult={testResult}
      />
    </div>
  );
}
