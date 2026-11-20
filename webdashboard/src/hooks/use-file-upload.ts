import { useMutation } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import type { FileRecord } from '@/lib/api/types';

import { client } from '@/lib/api/client';
import { components } from '@/types/api-generated';
import { computeSHA256 } from '@/utils/hash';

export interface UseFileUploadOptions {
  category: string;
  onError?: (error: Error) => void;
  onSuccess?: (file: FileRecord) => void;
  ownerId: string;
  ownerType: string;
}

export function useFileUpload({
  category,
  ownerType,
  ownerId,
  onSuccess,
  onError,
}: UseFileUploadOptions) {
  const { t } = useTranslation();
  const [progress, setProgress] = useState(0);

  const uploadMutation = useMutation({
    mutationFn: async (file: File) => {
      setProgress(0);
      const isLargeFile = file.size > 5 * 1024 * 1024; // 5MB threshold
      const contentHash = await computeSHA256(file);

      if (!isLargeFile) {
        // Direct buffer upload
        const formData = new FormData();
        formData.append('file', file);
        formData.append('category', category);
        formData.append('owner_type', ownerType);
        formData.append('owner_id', ownerId);
        formData.append('content_hash', contentHash); // Ready for backend support

        // FormData upload: the generated schema body type is incompatible with
        // FormData at runtime, so we bridge the body with `never` (output
        // position). openapi-fetch's defaultBodySerializer handles FormData
        // natively, so no custom serializer is needed. See coding-standard §10.1.
        try {
          const { data } = await client.POST<FileRecord>('/api/v1/files/upload', {
            body: formData,
          });
          setProgress(100);
          return data;
        } catch (error) {
          const message = error instanceof Error && error.message ? error.message : t("errors.upload_failed");
          throw new Error(message);
        }
      } else {
        // Presigned URL upload uses typed backend request and response contracts.
        let presignedData: components["schemas"]["CreateUploadUrlResponse"];
        try {
          const { data } = await client.POST<components["schemas"]["CreateUploadUrlResponse"]>(
            "/api/v1/files/presigned",
            {
              body: {
                category: category as components["schemas"]["FileCategory"],
                file_name: file.name,
                file_size: file.size,
                mime_type: file.type || "application/octet-stream",
                owner_id: ownerId,
                owner_type: ownerType as components["schemas"]["FileOwnerType"],
              } satisfies components["schemas"]["CreateUploadUrlRequest"],
            },
          );
          if (!data?.file || !data.upload_url) throw new Error(t("errors.presigned_url_failed"));
          presignedData = data;
        } catch (err) {
          const uploadErr: { message?: string } | null | undefined = err as { message?: string } | null | undefined;
          throw new Error(uploadErr?.message ?? t("errors.presigned_url_failed"));
        }

        const { file: dbFile, upload_url } = presignedData;

        // Use XMLHttpRequest for accurate upload progress tracking to S3
        await new Promise((resolve, reject) => {
            const xhr = new XMLHttpRequest();
            xhr.open('PUT', upload_url, true);
            xhr.setRequestHeader('Content-Type', file.type || 'application/octet-stream');

            xhr.upload.onprogress = (e) => {
                if (e.lengthComputable) {
                    setProgress(Math.round((e.loaded / e.total) * 100));
                }
            };

            xhr.onload = () => {
                if (xhr.status >= 200 && xhr.status < 300) {
                    resolve(true);
                } else {
                    reject(new Error(t('errors.s3_upload_failed')));
                }
            };

            xhr.onerror = () => { reject(new Error(t('errors.network_error'))); };
            xhr.send(file);
        });

        return dbFile;
      }
    },
    onSuccess: (data) => {
      onSuccess?.(data);
    },
    onError: (err) => {
      onError?.(err);
    }
  });

  return {
    upload: uploadMutation.mutate,
    uploadAsync: uploadMutation.mutateAsync,
    isUploading: uploadMutation.isPending,
    progress,
    error: uploadMutation.error,
  };
}
