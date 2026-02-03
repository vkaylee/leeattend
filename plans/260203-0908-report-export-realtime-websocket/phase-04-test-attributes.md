# Phase 4: Add Test Attributes

**Status**: pending
**Priority**: P2
**Effort**: 0.5h

---

## Context

Add `data-testid` attributes to all interactive and testable elements in `ReportExportDialog` for E2E testing.

---

## Requirements

### Functional
- Add `data-testid` to all buttons
- Add `data-testid` to progress indicators
- Add `data-testid` to status messages
- Add `data-testid` to error/notification displays

### Non-Functional
- Use kebab-case naming
- Follow pattern: `component-element-type`
- No visual changes

---

## Related Files

**Modify**:
- `webdashboard/src/components/features/reports/ReportExportDialog.tsx`

---

## Implementation Steps

### 1. Modal Container

**Location**: Line 121-128

```typescript
<Modal
  isOpen={isOpen}
  onClose={onClose}
  title={t("report_export.title")}
  subtitle={t("report_export.subtitle")}
  icon={<FileSpreadsheet size={24} />}
  maxWidth="md"
  data-testid="report-export-dialog"  // ADD THIS
>
```

### 2. Export Info Section

**Location**: Line 131 (info container)

```typescript
<div
  className="bg-slate-50 rounded-2xl p-4 space-y-2"
  data-testid="report-export-info"  // ADD THIS
>
```

### 3. Progress Bar

**Location**: Line 152 (progress container)

```typescript
<div
  className="h-4 w-full bg-slate-100 rounded-full overflow-hidden shadow-inner"
  data-testid="report-export-progress-bar"  // ADD THIS
>
  <div
    className={cn(...)}
    style={{ width: `${phase === "completed" ? 100 : percentage}%` }}
    data-testid="report-export-progress-fill"  // ADD THIS
  />
</div>
```

### 4. Status Text

**Location**: Line 176 (status span)

```typescript
<span
  className={cn(...)}
  data-testid="report-export-status-text"  // ADD THIS
>
  {phase === "starting" && t("report_export.status.preparing")}
  {/* ... */}
</span>
```

### 5. Batch Counter

**Location**: Line 192 (batch progress text)

```typescript
<p
  className="text-center text-sm text-slate-500"
  data-testid="report-export-batch-counter"  // ADD THIS
>
  {t("report_export.batches_completed", {...})}
</p>
```

### 6. Error Message

**Location**: Line 202 (error container)

```typescript
<div
  className="bg-rose-50 border border-rose-200 rounded-xl p-4"
  data-testid="report-export-error"  // ADD THIS
>
  <p className="text-sm text-rose-700">{error}</p>
</div>
```

### 7. Fallback Notification

**Location**: After error message (Phase 3 addition)

```typescript
<div
  className="bg-amber-50 border border-amber-200 rounded-xl p-3"
  data-testid="report-export-fallback-notice"  // ADD THIS
>
  <p className="text-xs text-amber-700">
    ⚠️ {t("report_export.fallback_mode")}
  </p>
</div>
```

### 8. Action Buttons

**Location**: Lines 211-281 (all buttons)

**Cancel Button** (idle phase):
```typescript
<button
  onClick={onClose}
  className="..."
  data-testid="report-export-cancel-button"  // ADD THIS
>
  {t("common.cancel")}
</button>
```

**Start Export Button** (idle phase):
```typescript
<button
  onClick={handleStartExport}
  className="..."
  data-testid="report-export-start-button"  // ADD THIS
>
  <Download size={18} />
  {t("report_export.start_export")}
</button>
```

**Run in Background Button** (processing phase):
```typescript
<button
  onClick={onClose}
  className="..."
  data-testid="report-export-background-button"  // ADD THIS
>
  {t("report_export.run_background")}
</button>
```

**Close Button** (completed phase):
```typescript
<button
  onClick={onClose}
  className="..."
  data-testid="report-export-close-button"  // ADD THIS
>
  {t("common.close")}
</button>
```

**Download Button** (completed phase):
```typescript
<button
  onClick={handleDownload}
  className="..."
  data-testid="report-export-download-button"  // ADD THIS
>
  <Download size={18} />
  {t("report_export.download_button")}
</button>
```

**Retry Button** (failed phase):
```typescript
<button
  onClick={handleStartExport}
  className="..."
  data-testid="report-export-retry-button"  // ADD THIS
>
  {t("common.retry")}
</button>
```

**Downloading Button** (downloading phase):
```typescript
<button
  disabled
  className="..."
  data-testid="report-export-downloading-button"  // ADD THIS
>
  <Loader2 size={18} className="animate-spin" />
  {t("report_export.status.downloading")}
</button>
```

---

## Complete Test ID List

| Element | Test ID | Usage |
|---------|---------|-------|
| Modal container | `report-export-dialog` | Dialog visibility |
| Info section | `report-export-info` | Verify metadata |
| Progress bar | `report-export-progress-bar` | Progress tracking |
| Progress fill | `report-export-progress-fill` | Visual progress |
| Status text | `report-export-status-text` | Phase verification |
| Batch counter | `report-export-batch-counter` | Batch progress |
| Error message | `report-export-error` | Error validation |
| Fallback notice | `report-export-fallback-notice` | Fallback detection |
| Cancel button | `report-export-cancel-button` | Dialog dismiss |
| Start button | `report-export-start-button` | Initiate export |
| Background button | `report-export-background-button` | Background mode |
| Close button | `report-export-close-button` | Post-completion |
| Download button | `report-export-download-button` | File download |
| Retry button | `report-export-retry-button` | Error recovery |
| Downloading button | `report-export-downloading-button` | Download state |

---

## Todo List

- [ ] Add `data-testid` to modal container
- [ ] Add `data-testid` to info section
- [ ] Add `data-testid` to progress bar + fill
- [ ] Add `data-testid` to status text
- [ ] Add `data-testid` to batch counter
- [ ] Add `data-testid` to error message
- [ ] Add `data-testid` to fallback notice
- [ ] Add `data-testid` to all buttons (7 total)
- [ ] Verify no TypeScript errors
- [ ] Document test IDs in E2E test file

---

## Success Criteria

✅ All interactive elements have `data-testid`
✅ Naming follows kebab-case pattern
✅ No visual changes
✅ TypeScript compiles without errors
✅ Test IDs documented for E2E phase

---

## Testing Notes

**Verification**:
1. Build project: `npm run build`
2. Inspect elements in browser DevTools
3. Verify `data-testid` attributes present
4. Check naming consistency

**Example E2E selector**:
```typescript
await page.getByTestId('report-export-start-button').click();
await expect(page.getByTestId('report-export-status-text')).toContainText('Processing');
```

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Typo in test ID | Low | Consistent naming pattern |
| Missing elements | Medium | Comprehensive checklist |
| Conflicting IDs | Low | Unique, descriptive names |

---

## Security Considerations

- `data-testid` attributes are harmless (no security impact)
- Not visible to end users (dev tools only)

---

## Next Steps

→ Phase 5: Write unit tests
