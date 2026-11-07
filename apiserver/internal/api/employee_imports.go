package api

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

    "github.com/google/uuid"
    "github.com/vkaylee/leeattend/apiserver/internal/apperror"
    "github.com/vkaylee/leeattend/apiserver/internal/auth"
    httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)


var terminationReasons = map[string]struct{}{
	"voluntary_resignation": {}, "involuntary_termination": {}, "end_of_contract": {}, "retirement": {},
	"mutual_agreement": {}, "abandonment": {}, "death": {}, "other": {},
}

type EmployeeImportJob struct {
	JobID       uuid.UUID `json:"job_id"`
	WorkspaceID uuid.UUID `json:"workspace_id"`
	UserID      uuid.UUID `json:"user_id"`
	FilePath    string    `json:"file_path"`
}
type TerminationJobRow struct {
	EmploymentID      uuid.UUID `json:"employment_id"`
	TerminationDate   string    `json:"termination_date"`
	TerminationReason string    `json:"termination_reason"`
	TerminationNotes  *string   `json:"termination_notes,omitempty"`
}
type TerminationImportJob struct {
	JobID       uuid.UUID           `json:"job_id"`
	WorkspaceID uuid.UUID           `json:"workspace_id"`
	UserID      uuid.UUID           `json:"user_id"`
	Rows        []TerminationJobRow `json:"rows"`
}

type TerminationEmployment struct {
	ID        uuid.UUID
	Status    string
	StartDate time.Time
}
type TerminationLeaveBalance struct {
	LeaveTypeID   uuid.UUID
	LeaveTypeCode string
	LeaveTypeName string
	Entitlement   int
}
type TerminationCandidate struct {
	EmployeeID   uuid.UUID
	EmployeeCode string
	FullName     string
	Employments  []TerminationEmployment
	Balances     map[int][]TerminationLeaveBalance
}

type EmployeeImportRepository interface {
	CreateEmployeeImportJob(context.Context, uuid.UUID, uuid.UUID, string, uuid.UUID) error
	TerminationCandidates(context.Context, uuid.UUID, []string) ([]TerminationCandidate, error)
	TerminationStatuses(context.Context, uuid.UUID, []uuid.UUID) (map[uuid.UUID]string, error)
}
type EmployeeImportQueue interface {
	EnqueueEmployeeImport(context.Context, EmployeeImportJob) error
	EnqueueTerminationImport(context.Context, TerminationImportJob) error
}

type ValidTerminationImportRow struct {
	RowNumber         int                    `json:"row_number"`
	EmployeeID        uuid.UUID              `json:"employee_id"`
	EmployeeCode      string                 `json:"employee_code"`
	FullName          string                 `json:"full_name"`
	StartDate         string                 `json:"start_date"`
	TerminationDate   string                 `json:"termination_date"`
	EmploymentID      uuid.UUID              `json:"employment_id"`
	ProratedLeaves    []ProratedLeavePreview `json:"prorated_leaves"`
	TerminationReason string                 `json:"termination_reason"`
	TerminationNotes  *string                `json:"termination_notes"`
	Warnings          []string               `json:"warnings"`
}
type InvalidTerminationImportRow struct {
	RowNumber          int     `json:"row_number"`
	EmployeeCode       *string `json:"employee_code"`
	RawTerminationDate *string `json:"raw_termination_date"`
	Reason             string  `json:"reason"`
}
type TerminationImportPreviewResponse struct {
	ValidRows   []ValidTerminationImportRow   `json:"valid_rows"`
	InvalidRows []InvalidTerminationImportRow `json:"invalid_rows"`
	CanCommit   bool                          `json:"can_commit"`
}
type ProratedLeavePreview struct {
	LeaveTypeID    uuid.UUID `json:"leave_type_id"`
	LeaveTypeCode  string    `json:"leave_type_code"`
	LeaveTypeName  string    `json:"leave_type_name"`
	OldEntitlement int       `json:"old_entitlement"`
	NewEntitlement int       `json:"new_entitlement"`
}
type ConfirmTerminationImportRow struct {
	EmploymentID      uuid.UUID `json:"employment_id"`
	TerminationDate   string    `json:"termination_date"`
	TerminationReason string    `json:"termination_reason"`
	TerminationNotes  *string   `json:"termination_notes"`
}
type ConfirmTerminationImportRequest struct {
	Rows []ConfirmTerminationImportRow `json:"rows"`
}
type ConfirmTerminationImportResponse struct {
	SuccessCount int       `json:"success_count"`
	JobID        uuid.UUID `json:"job_id"`
}
type EmployeeImportHandler struct {
	Repository EmployeeImportRepository
	Queue      EmployeeImportQueue
	Authorizer EmployeeAuthorizer
	StorageDir string
}

func (h EmployeeImportHandler) allowed(r *http.Request, workspaceID uuid.UUID, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, workspaceID, permission)
}
func (h EmployeeImportHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}
func (h EmployeeImportHandler) fail(w http.ResponseWriter, r *http.Request, err error) {
	var appErr *apperror.AppError
	if errors.As(err, &appErr) {
		apperror.WriteError(w, r, appErr)
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
func (h EmployeeImportHandler) userID(r *http.Request) (uuid.UUID, bool) {
	return httpinfra.UserIDFromContext(r.Context())
}

func (h EmployeeImportHandler) Upload(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	userID, userOK := h.userID(r)
    if !ok || !userOK || !h.allowed(r, workspaceID, string(auth.PermissionEmployeesUpload)) {
		h.forbidden(w, r)
		return
	}
	fileName, content, err := readExcelMultipart(r)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	storageDir := h.StorageDir
	if storageDir == "" {
		storageDir = "storage"
	}
	if err := os.MkdirAll(filepath.Join(storageDir, "imports"), 0o750); err != nil {
		h.fail(w, r, err)
		return
	}
	jobID := uuid.New()
	filePath := filepath.Join(storageDir, "imports", jobID.String()+"_"+filepath.Base(fileName))
	if err := os.WriteFile(filePath, content, 0o640); err != nil {
		h.fail(w, r, err)
		return
	}
	if h.Repository == nil || h.Queue == nil {
		h.fail(w, r, errors.New("employee import dependencies are not configured"))
		return
	}
	if err := h.Repository.CreateEmployeeImportJob(r.Context(), workspaceID, userID, filePath, jobID); err != nil {
		h.fail(w, r, err)
		return
	}
	if err := h.Queue.EnqueueEmployeeImport(r.Context(), EmployeeImportJob{JobID: jobID, WorkspaceID: workspaceID, UserID: userID, FilePath: filePath}); err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, struct {
		JobID uuid.UUID `json:"job_id"`
	}{jobID})
}

func (h EmployeeImportHandler) PreviewTermination(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
    if !ok || !h.allowed(r, workspaceID, string(auth.PermissionEmployeesUpdate)) {
		h.forbidden(w, r)
		return
	}
	_, content, err := readExcelMultipart(r)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	mapped, err := parseEmployeeWorkbook(content, []string{"employee_code", "termination_date", "termination_reason", "termination_notes"})
	if err != nil {
		h.fail(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	response, err := h.buildTerminationPreview(r.Context(), workspaceID, mapped)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, response)
}

func (h EmployeeImportHandler) buildTerminationPreview(ctx context.Context, workspaceID uuid.UUID, mapped []map[string]string) (TerminationImportPreviewResponse, error) {
	response := TerminationImportPreviewResponse{ValidRows: make([]ValidTerminationImportRow, 0), InvalidRows: make([]InvalidTerminationImportRow, 0)}
	codes := make([]string, 0, len(mapped))
	parsed := make([]terminationParsedRow, 0, len(mapped))
	seen := make(map[string]struct{}, len(mapped))
	for _, row := range mapped {
		rowNumber := parseRowNumber(row["_row_number"])
		code := strings.TrimSpace(row["employee_code"])
		rawDate := strings.TrimSpace(row["termination_date"])
		reason := strings.ToLower(strings.TrimSpace(row["termination_reason"]))
		invalid := func(message string) {
			codeCopy, dateCopy := code, rawDate
			response.InvalidRows = append(response.InvalidRows, InvalidTerminationImportRow{RowNumber: rowNumber, EmployeeCode: &codeCopy, RawTerminationDate: &dateCopy, Reason: message})
		}
		if code == "" {
			invalid("employee code is required")
			continue
		}
		date, dateErr := parseTerminationDate(rawDate)
		if dateErr != nil {
			invalid("invalid date format")
			continue
		}
		if _, valid := terminationReasons[reason]; !valid {
			invalid("invalid termination reason")
			continue
		}
		normalized := strings.ToUpper(code)
		if _, duplicate := seen[normalized]; duplicate {
			invalid("duplicate employee code in the file")
			continue
		}
		seen[normalized] = struct{}{}
		codes = append(codes, normalized)
		parsed = append(parsed, terminationParsedRow{rowNumber, code, normalized, rawDate, date, reason, optionalString(row["termination_notes"])})
	}
	if len(parsed) == 0 {
		return response, nil
	}
	if h.Repository == nil {
		return response, errors.New("employee import repository is not configured")
	}
	candidates, err := h.Repository.TerminationCandidates(ctx, workspaceID, codes)
	if err != nil {
		return response, err
	}
	byCode := make(map[string]TerminationCandidate, len(candidates))
	for _, candidate := range candidates {
		byCode[strings.ToUpper(strings.TrimSpace(candidate.EmployeeCode))] = candidate
	}
	for _, row := range parsed {
		candidate, found := byCode[row.normalizedCode]
		if !found {
			response.InvalidRows = append(response.InvalidRows, InvalidTerminationImportRow{RowNumber: row.rowNumber, EmployeeCode: &row.code, RawTerminationDate: &row.rawDate, Reason: "employee not found"})
			continue
		}
		employments := append([]TerminationEmployment(nil), candidate.Employments...)
		sort.Slice(employments, func(i, j int) bool { return employments[i].StartDate.After(employments[j].StartDate) })
		active := make([]TerminationEmployment, 0, len(employments))
		for _, employment := range employments {
			if employment.Status != "terminated" {
				active = append(active, employment)
			}
		}
		if len(active) == 0 {
			message := "employee has no active employment"
			if len(employments) > 0 {
				message = "employee is already terminated"
			}
			response.InvalidRows = append(response.InvalidRows, InvalidTerminationImportRow{RowNumber: row.rowNumber, EmployeeCode: &row.code, RawTerminationDate: &row.rawDate, Reason: message})
			continue
		}
		chosen := active[0]
		if row.date.Before(chosen.StartDate) {
			response.InvalidRows = append(response.InvalidRows, InvalidTerminationImportRow{RowNumber: row.rowNumber, EmployeeCode: &row.code, RawTerminationDate: &row.rawDate, Reason: "termination date cannot be before employment start date (" + chosen.StartDate.Format("02/01/2006") + ")"})
			continue
		}
		warnings := make([]string, 0)
		if len(active) > 1 {
			warnings = append(warnings, "multiple non-terminated employments; latest employment selected")
		}
		leaves := make([]ProratedLeavePreview, 0)
		for _, balance := range candidate.Balances[row.date.Year()] {
			startOfYear := time.Date(row.date.Year(), 1, 1, 0, 0, 0, 0, time.UTC)
			endOfYear := time.Date(row.date.Year(), 12, 31, 0, 0, 0, 0, time.UTC)
			effectiveStart := chosen.StartDate
			if effectiveStart.Before(startOfYear) {
				effectiveStart = startOfYear
			}
			days := int(row.date.Sub(effectiveStart).Hours()/24) + 1
			total := int(endOfYear.Sub(startOfYear).Hours()/24) + 1
			newEntitlement := 0
			if days > 0 {
				newEntitlement = (balance.Entitlement*days + total/2) / total
			}
			leaves = append(leaves, ProratedLeavePreview{LeaveTypeID: balance.LeaveTypeID, LeaveTypeCode: balance.LeaveTypeCode, LeaveTypeName: balance.LeaveTypeName, OldEntitlement: balance.Entitlement, NewEntitlement: newEntitlement})
		}
		response.ValidRows = append(response.ValidRows, ValidTerminationImportRow{RowNumber: row.rowNumber, EmployeeID: candidate.EmployeeID, EmployeeCode: row.code, FullName: candidate.FullName, StartDate: chosen.StartDate.Format("2006-01-02"), TerminationDate: row.date.Format("2006-01-02"), EmploymentID: chosen.ID, ProratedLeaves: leaves, TerminationReason: row.reason, TerminationNotes: row.notes, Warnings: warnings})
	}
	response.CanCommit = len(response.ValidRows) > 0
	return response, nil
}

type terminationParsedRow struct {
	rowNumber                     int
	code, normalizedCode, rawDate string
	date                          time.Time
	reason                        string
	notes                         *string
}

func parseRowNumber(value string) int {
	n, err := strconvAtoi(value)
	if err != nil || n < 1 {
		return 2
	}
	return n
}
func strconvAtoi(value string) (int, error) {
	var n int
	_, err := fmtSscanf(value, "%d", &n)
	return n, err
}
func fmtSscanf(value, format string, target *int) (int, error) {
	return fmt.Sscanf(value, format, target)
}
func optionalString(value string) *string {
	value = strings.TrimSpace(value)
	if value == "" {
		return nil
	}
	return &value
}
func parseTerminationDate(value string) (time.Time, error) {
	for _, layout := range []string{"2006-01-02", "02/01/2006", "01/02/2006", "2006/01/02"} {
		if parsed, err := time.Parse(layout, value); err == nil {
			return parsed, nil
		}
	}
	return time.Time{}, errors.New("invalid date")
}

func (h EmployeeImportHandler) ConfirmTermination(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	userID, userOK := h.userID(r)
	if !ok || !userOK || !h.allowed(r, workspaceID, EmployeeImportUpdatePermission) {
		h.forbidden(w, r)
		return
	}
	request, err := DecodeJSON[ConfirmTerminationImportRequest](r, 10<<20)
	if err != nil || len(request.Rows) == 0 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Repository == nil || h.Queue == nil {
		h.fail(w, r, errors.New("employee import dependencies are not configured"))
		return
	}
	ids := make([]uuid.UUID, len(request.Rows))
	rows := make([]TerminationJobRow, len(request.Rows))
	for i, row := range request.Rows {
		if row.EmploymentID == uuid.Nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		date, dateErr := parseTerminationDate(row.TerminationDate)
		if dateErr != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		if _, valid := terminationReasons[strings.ToLower(strings.TrimSpace(row.TerminationReason))]; !valid {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		ids[i] = row.EmploymentID
		rows[i] = TerminationJobRow{EmploymentID: row.EmploymentID, TerminationDate: date.Format("2006-01-02"), TerminationReason: strings.ToLower(strings.TrimSpace(row.TerminationReason)), TerminationNotes: row.TerminationNotes}
	}
	statuses, err := h.Repository.TerminationStatuses(r.Context(), workspaceID, ids)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	for _, id := range ids {
		if statuses[id] == "terminated" {
			apperror.WriteError(w, r, apperror.Validation("error.employment_inactive"))
			return
		}
	}
	jobID := uuid.New()
	if err := h.Queue.EnqueueTerminationImport(r.Context(), TerminationImportJob{JobID: jobID, WorkspaceID: workspaceID, UserID: userID, Rows: rows}); err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, ConfirmTerminationImportResponse{SuccessCount: len(rows), JobID: jobID})
}

func (h EmployeeImportHandler) TerminationTemplate(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, EmployeeImportUpdatePermission) {
		h.forbidden(w, r)
		return
	}
	data, err := generateTerminationTemplate(r.URL.Query().Get("lang"))
	if err != nil {
		h.fail(w, r, err)
		return
	}
	w.Header().Set("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
	w.Header().Set("Content-Disposition", `attachment; filename="employee_termination_template.xlsx"`)
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(data)
}

func readExcelMultipart(r *http.Request) (string, []byte, error) {
	if r.ContentLength > maxEmployeeImportBytes {
		return "", nil, apperror.PayloadTooLarge(apperror.KeyUploadFileTooLarge)
	}
	reader, err := r.MultipartReader()
	if err != nil {
		return "", nil, apperror.Validation(apperror.KeyValidation)
	}
	for {
		part, nextErr := reader.NextPart()
		if errors.Is(nextErr, io.EOF) {
			break
		}
		if nextErr != nil {
			return "", nil, apperror.Validation(apperror.KeyValidation)
		}
		if part.FormName() != "file" {
			continue
		}
		filename := filepath.Base(part.FileName())
		if filename == "." || filename == "" {
			return "", nil, apperror.Validation(apperror.KeyValidation)
		}
		extension := strings.ToLower(filepath.Ext(filename))
		if extension != ".xlsx" && extension != ".xls" {
			return "", nil, apperror.Validation("error.invalid_file_format")
		}
		limited := io.LimitReader(part, maxEmployeeImportBytes+1)
		content, readErr := io.ReadAll(limited)
		if readErr != nil {
			return "", nil, apperror.Validation(apperror.KeyValidation)
		}
		if int64(len(content)) > maxEmployeeImportBytes {
			return "", nil, apperror.PayloadTooLarge(apperror.KeyUploadFileTooLarge)
		}
		if !isXLSX(content) && !isXLS(content) {
			return "", nil, apperror.Validation("error.invalid_file_format")
		}
		if extension == ".xlsx" && !isXLSX(content) || extension == ".xls" && !isXLS(content) {
			return "", nil, apperror.Validation("error.invalid_file_format")
		}
		return filename, content, nil
	}
	return "", nil, apperror.Validation(apperror.KeyValidation)
}
