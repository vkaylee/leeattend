# Phase 4: Document Management System

**Priority**: Medium
**Status**: Pending
**Estimated Effort**: 3-4 hours

---

## Context Links

- Backend storage: Local filesystem initially
- Frontend upload: React Dropzone pattern
- Existing import: `webdashboard/src/components/AttendanceImportDialog.tsx`

## Overview

Implement file storage, upload, and download system for employee documents.

## Key Insights

- Start with local filesystem storage (./uploads/)
- Future migration to S3/R2 possible
- File validation critical (type, size, virus)
- Signed URLs for download (time-limited)

## Requirements

### Functional
- Upload documents (PDF, images, DOC)
- Download with streaming
- Delete documents (soft delete)
- Document categorization (ID, Contract, Certificate, Other)
- Expiry date tracking with warnings

### Non-Functional
- Max file size: 10MB
- Allowed types: PDF, PNG, JPG, JPEG, DOC, DOCX
- Storage path: ./uploads/{workspace_id}/{employee_id}/
- Cleanup job for expired soft-deleted files

## Architecture

```
File Upload Flow:
1. Frontend: Select file → Validate type/size → FormData POST
2. Backend: Receive multipart → Validate → Save to disk → Save metadata
3. Response: Document metadata with ID

File Download Flow:
1. Frontend: Request /documents/:id/download
2. Backend: Validate ownership → Generate signed URL or stream
3. Response: File stream with proper headers
```

## Implementation Steps

### 1. Backend Storage Service
```rust
// services/storage.rs
pub struct StorageService {
    base_path: PathBuf,
}

impl StorageService {
    pub async fn save_file(&self, workspace_id: Uuid, employee_id: Uuid,
                           file_name: &str, data: &[u8]) -> Result<String, Error>;
    pub async fn get_file(&self, path: &str) -> Result<Vec<u8>, Error>;
    pub async fn delete_file(&self, path: &str) -> Result<(), Error>;
}
```

### 2. Upload Endpoint Enhancement
```rust
// handlers/ess.rs
pub async fn upload_document(
    State(state): State<AppState>,
    employee: EmployeeContext,
    mut multipart: Multipart,
) -> Result<Json<EmployeeDocument>, ApiError> {
    // 1. Extract file from multipart
    // 2. Validate file type and size
    // 3. Generate unique filename
    // 4. Save to storage
    // 5. Create database record
    // 6. Return document metadata
}
```

### 3. Download Endpoint
```rust
pub async fn download_document(
    State(state): State<AppState>,
    employee: EmployeeContext,
    Path(document_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Get document metadata
    // 2. Validate ownership
    // 3. Stream file with headers
}
```

### 4. Frontend Uploader Component
```typescript
// components/ess/DocumentUploader.tsx
export function DocumentUploader({ onUpload }: Props) {
    const [isDragging, setIsDragging] = useState(false);
    const [uploadProgress, setUploadProgress] = useState(0);

    const handleDrop = async (files: FileList) => {
        // Validate
        // Create FormData
        // Upload with progress
        // Notify parent
    };
}
```

### 5. Document Type Categories
```typescript
const DOCUMENT_TYPES = [
    { value: 'id_card', label: 'ID Card / Passport' },
    { value: 'contract', label: 'Employment Contract' },
    { value: 'certificate', label: 'Certificate / Diploma' },
    { value: 'tax', label: 'Tax Document' },
    { value: 'other', label: 'Other' },
];
```

## Todo List

- [ ] Create services/storage.rs
- [ ] Implement multipart upload handler
- [ ] Implement download streaming
- [ ] Add file validation (type, size)
- [ ] Create uploads directory structure
- [ ] Create DocumentUploader component
- [ ] Add document type selector
- [ ] Add expiry date picker
- [ ] Create document list with actions
- [ ] Add delete confirmation

## Success Criteria

- Files upload and save correctly
- Downloads work with proper MIME types
- Large files stream without memory issues
- Invalid files rejected with clear message
- Expiring documents show warning badge

## Security Considerations

- Validate file content matches extension (magic bytes)
- Sanitize filenames (prevent path traversal)
- Rate limit uploads (5 per hour)
- Scan for malware (future enhancement with ClamAV)

## Related Code Files

- Create: `apiserver/src/services/storage.rs`
- Modify: `apiserver/src/handlers/ess.rs`
- Create: `webdashboard/src/components/ess/DocumentUploader.tsx`
- Create: `webdashboard/src/components/ess/DocumentCard.tsx`

## Next Steps

After document system ready, proceed to Phase 5: Testing
