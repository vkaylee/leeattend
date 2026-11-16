# Bulk OT CSV Upload Feature

## Overview
Allows Admin users to upload multiple paper-based OT claims via CSV file, reducing data entry time from 8-16 hours to 30 minutes for 1000 entries.

## Endpoint
```
POST /api/v1/attendance/bulk-upload-paper-ot
```

## Authentication
- **Authorization**: Admin role required
- **Headers**: Standard organization authentication

## Request Format
- **Content-Type**: `multipart/form-data`
- **Field Name**: `file`
- **File Type**: CSV (.csv)
- **Max File Size**: 10 MB
- **Max Rows**: 10,000

## CSV Format

### Required Columns
| Column | Type | Format | Description |
|--------|------|--------|-------------|
| employee_id | String | E001, E002, etc. | Employee ID number |
| date | Date | YYYY-MM-DD | Date of OT work |
| minutes | Integer | > 0 | OT minutes claimed |
| reason | String | Non-empty | Reason for OT claim |

### Example CSV
```csv
employee_id,date,minutes,reason
E001,2026-01-27,120,Production line overtime - paper form #123
E002,2026-01-27,90,Urgent order completion
E003,2026-01-27,60,Machine maintenance work
```

## Validation Rules

### Row-Level Validation
- ✅ Employee ID must not be empty
- ✅ Employee must exist in organization
- ✅ Date must be in `YYYY-MM-DD` format
- ✅ Date cannot be in the future
- ✅ Minutes must be > 0
- ✅ Minutes must not exceed `system_detected_ot_minutes` (if summary exists)
- ✅ Reason must not be empty
- ✅ No duplicate `(employee_id, date)` in same upload

### Processing Mode
- **Partial Success (default)**: Inserts valid rows, skips invalid, returns detailed errors
- Continues processing even if some rows fail
- All successful entries are committed to database

## Response Format

### Success Response (HTTP 200)
```json
{
  "total_rows": 3,
  "success_count": 2,
  "failed_count": 1,
  "errors": [
    {
      "line_number": 3,
      "employee_id": "E999",
      "error_message": "Employee not found in organization"
    }
  ]
}
```

### Error Response

#### HTTP 400: Bad Request
- Invalid CSV format
- Missing file in multipart form

#### HTTP 403: Forbidden
- User is not Admin or Owner

#### HTTP 413: Payload Too Large
- File size > 10 MB
- Row count > 10,000

#### HTTP 500: Internal Server Error
- Database errors
- System failures

## Error Types

| Error Message | Cause | Solution |
|---------------|-------|----------|
| Employee ID is required | Empty employee_id field | Provide valid employee ID |
| Invalid date format | Date not in YYYY-MM-DD | Use correct date format |
| Date cannot be in the future | Date > today | Use past or current date |
| Minutes must be greater than 0 | minutes <= 0 | Provide positive integer |
| Reason is required | Empty reason field | Provide non-empty reason |
| Employee not found | Invalid employee_id | Check employee exists |
| Claimed X minutes exceeds system detected Y minutes | claimed > detected | Reduce claimed minutes or check attendance |
| Duplicate entry in this upload | Same (employee_id, date) appears twice | Remove duplicates |

## Business Rules

1. **OT Status Transition**:
   - Upon successful upload, `ot_status` → `pending_manager`
   - `claimed_ot_minutes` is set to the value from CSV
   - `entered_via` is set to `"paper_form"`

2. **System Detected OT**:
   - If attendance summary exists with `system_detected_ot_minutes`, claimed minutes must not exceed it
   - If no summary exists, paper form creates new entry without validation against detected OT

3. **Metadata Tracking**:
   - `claim_entered_by_user_id`: Set to current admin user
   - `claim_entered_at`: Set to current timestamp
   - `paper_form_photo_url`: Not set for bulk upload (single entry only)

## Performance

- **Single Entry**: 30-60 seconds per entry (dialog-based)
- **Bulk Upload**: 30 minutes for 1000 entries
- **Speedup**: 16-32x faster

## Use Cases

### Factory Scenario
- **Input**: Paper OT forms from 500 employees
- **Process**:
  1. Admin prepares CSV from paper forms
  2. Admin uploads CSV via dashboard
  3. System validates and processes entries
  4. Admin reviews error report (if any)
  5. Manager approves claims via OT approval interface

### Data Migration
- Import historical OT claims from legacy systems
- Bulk correction of OT entries

## Implementation Details

### Code Structure
```
apiserver/
├── internal/api/          # HTTP handlers and request validation
├── internal/repository/   # Attendance and overtime persistence
└── internal/domain/       # Overtime and attendance rules
```

### Key Features
- ✅ Row-by-row validation without stopping on errors
- ✅ Duplicate detection within upload
- ✅ Reuse of existing attendance/overtime persistence rules
- ✅ RBAC enforcement
- ✅ Comprehensive error reporting

### Testing
- API unit and integration tests cover valid uploads and validation failures.

## Security Considerations

1. **Access Control**: Admin role enforced at handler level
2. **File Size Limits**: Prevents memory exhaustion attacks
3. **Row Count Limits**: Prevents database overload
4. **CSV Injection**: Input is not directly executed or rendered
5. **No Photo URL**: Bulk upload doesn't support photo attachments (by design)

## Future Enhancements

- [ ] All-or-nothing transaction mode (configurable)
- [ ] Email notification to managers after bulk upload
- [ ] CSV template download from UI
- [ ] Progress tracking for large uploads (async processing)
- [ ] Audit log for bulk operations
- [ ] Support for bulk photo upload (zip file with references)
