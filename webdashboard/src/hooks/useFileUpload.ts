import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { client } from '@/lib/api/client';
import { useTranslation } from 'react-i18next';
import { components } from '@/types/api-generated';
import { computeSHA256 } from '@/utils/hash';

type WorkspaceFile = components['schemas']['WorkspaceFile'];

export interface UseFileUploadOptions {
  workspaceId: string;
  category: string;
  ownerType: string;
  ownerId: string;
  onSuccess?: (file: WorkspaceFile) => void;
  onError?: (error: Error) => void;
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
            // @ts-ignore - bypass body type check for FormData
            body: formData,
            // @ts-ignore
            bodySerializer: (body) => body,
        });

        if (error) {
            throw new Error((error as any).message || t('errors.upload_failed'));
        }
        
        setProgress(100);
        return data as WorkspaceFile;
      } else {
        // Presigned URL upload
        const { data: presignedData, error: presignedError } = await client.POST('/v1/workspaces/{ws_id}/files/presigned', {
            params: { path: { ws_id: workspaceId } },
            body: {
                file_name: file.name,
                mime_type: file.type || 'application/octet-stream',
                file_size: file.size,
                // @ts-ignore
                category,
                // @ts-ignore
                owner_type: ownerType,
                owner_id: ownerId,
                // @ts-ignore - Ready for backend support
                content_hash: contentHash,
            }
        });

        if (presignedError || !presignedData) {
            throw new Error((presignedError as any)?.message || t('errors.presigned_url_failed'));
        }

        const { upload_url, file: dbFile } = presignedData as any;

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
            
            xhr.onerror = () => reject(new Error(t('errors.network_error')));
            xhr.send(file);
        });

        return dbFile as WorkspaceFile;
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
