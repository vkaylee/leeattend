import { useMutation } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { client } from '@/lib/api/client';
import { components } from '@/types/api-generated';
import { computeSHA256 } from '@/utils/hash';

type WorkspaceFile = components['schemas']['WorkspaceFile'];
export interface UseFileUploadOptions {
  category: string;
  onError?: (error: Error) => void;
  onSuccess?: (file: WorkspaceFile) => void;
  ownerId: string;
  ownerType: string;
  workspaceId: string;
}

export function useFileUpload({
  workspaceId,
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

        const { data, error } = await client.POST('/v1/workspaces/{ws_id}/files/upload', {
            params: { path: { ws_id: workspaceId } },
            body: formData as unknown as string,
            bodySerializer: (body) => body as unknown as FormData,
        });

        if (error) {
            throw new Error((error as unknown as { message?: string }).message || t('errors.upload_failed'));
        }
        
        setProgress(100);
        return data;
      } else {
        // Presigned URL upload
        const { data: presignedData, error: presignedError } = await client.POST('/v1/workspaces/{ws_id}/files/presigned', {
            params: { path: { ws_id: workspaceId } },
            body: {
                category: category as components['schemas']['FileCategory'],
                content_hash: contentHash,
                file_name: file.name,
                file_size: file.size,
                mime_type: file.type || 'application/octet-stream',
                owner_id: ownerId,
                owner_type: ownerType as components['schemas']['FileOwnerType'],
            },
        });

        if (presignedError || !presignedData) {
            throw new Error((presignedError as unknown as { message?: string })?.message || t('errors.presigned_url_failed'));
        }

        const { file: dbFile, upload_url } = presignedData as unknown as { file: WorkspaceFile; upload_url: string };

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
