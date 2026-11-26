package repository

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"sort"
	"strings"
	"time"
)

var (
	ErrScheduleResolverUnavailable = errors.New("schedule lifecycle resolver cannot produce a complete sanitized result")
	ErrScheduleIdempotencyConflict = api.ErrScheduleIdempotencyConflict
	ErrSchedulePreviewStale        = api.ErrSchedulePreviewStale
	ErrScheduleAlreadyPublished    = api.ErrScheduleAlreadyPublished
	ErrScheduleRevisionStale       = api.ErrScheduleRevisionStale
	ErrScheduleNoCurrentRevision   = api.ErrScheduleNoCurrentRevision
	ErrScheduleLockedPeriod        = api.ErrScheduleLockedPeriod
	ErrSchedulePreviewScope        = api.ErrSchedulePreviewScope
	ErrScheduleTimezoneMismatch    = api.ErrScheduleTimezoneMismatch
)

const (
	lifecycleResolverVersion = "schedule-resolver-v1"
	previewTTL               = 15 * time.Minute
)

type lifecycleExpectedRevision struct {
	Key   string `json:"key"`
	Value int64  `json:"value"`
}

type lifecycleRequestHashInput struct {
	Operation          string                         `json:"operation"`
	ActorID            string                         `json:"actor_id"`
	EmploymentID       string                         `json:"employment_id,omitempty"`
	BusinessDate       string                         `json:"business_date,omitempty"`
	EmploymentIDs      []string                       `json:"employment_ids,omitempty"`
	StartDate          string                         `json:"start_date,omitempty"`
	EndDate            string                         `json:"end_date,omitempty"`
	Timezone           string                         `json:"timezone,omitempty"`
	Mode               string                         `json:"mode,omitempty"`
	ApprovalInstanceID *string                        `json:"approval_instance_id"`
	ExpectedRevisions  []lifecycleExpectedRevision    `json:"expected_revisions,omitempty"`
	ExpectedRevision   int64                          `json:"expected_revision,omitempty"`
	Reason             string                         `json:"reason"`
	Retroactive        bool                           `json:"retroactive,omitempty"`
	Replacement        *api.ScheduleReplanReplacement `json:"replacement,omitempty"`
	PreviewID          string                         `json:"preview_id,omitempty"`
	PreviewHash        string                         `json:"preview_hash,omitempty"`
	ConfirmDiffHash    string                         `json:"confirm_diff_hash,omitempty"`
}

type lifecycleRevision struct {
	ID                 uuid.UUID
	EmploymentID       uuid.UUID
	EmployerID         uuid.UUID
	BusinessDate       string
	Timezone           string
	RevisionNumber     int64
	State              string
	BatchID            uuid.UUID
	Mode               string
	ApprovalInstanceID *uuid.UUID
	Source             string
	SourceSnapshot     []byte
	PublishedAt        *time.Time
}

type lifecycleSegment struct {
	ID                    uuid.UUID
	SortOrder             int
	TemplateID            uuid.UUID
	ShiftVersionID        uuid.UUID
	StartAt               time.Time
	EndAt                 time.Time
	Breaks                json.RawMessage
	StartDayOffset        *int
	EndDayOffset          *int
	ShiftGroupKey         *string
	StartUTCOffsetMinutes *int
	EndUTCOffsetMinutes   *int
}

func (r ScheduleRepository) beginLifecycle(ctx context.Context) (TxBeginner, error) {
	db, err := requireDB(r.DB, "schedule lifecycle repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("schedule lifecycle repository: database does not support transactions")
	}
	return beginner, nil
}

// DBTX helpers keep writes inside the transaction and make rollback behavior
// testable with the same pgx abstractions used by other repositories.
func lockIdempotencyKey(ctx context.Context, tx DBTX, key string) error {
	_, err := tx.Exec(ctx, `SELECT pg_advisory_xact_lock(hashtextextended('schedule-lifecycle:' || $1, 0))`, key)
	return err
}

func loadBatchByIdempotency(ctx context.Context, tx DBTX, key string) (storedBatch, bool, error) {
	var batch storedBatch
	err := tx.QueryRow(ctx, `SELECT id, request_hash, requested_by, actor_type, actor_name, response_snapshot FROM schedule_publication_batches WHERE idempotency_key = $1`, key).Scan(&batch.ID, &batch.RequestHash, &batch.RequestedBy, &batch.ActorType, &batch.ActorName, &batch.ResponseSnapshot)
	if errors.Is(err, pgx.ErrNoRows) {
		return storedBatch{}, false, nil
	}
	return batch, err == nil, err
}

type storedBatch struct {
	ID               uuid.UUID
	RequestHash      string
	RequestedBy      *uuid.UUID
	ActorType        string
	ActorName        *string
	ResponseSnapshot []byte
}

func replayPublication(actorID uuid.UUID, requestHash string, stored storedBatch, response *api.SchedulePublicationResponse) error {
	if stored.ActorType != "human" || stored.RequestedBy == nil || *stored.RequestedBy != actorID {
		return api.ErrScheduleForbidden
	}
	if strings.TrimSpace(stored.RequestHash) != requestHash {
		return ErrScheduleIdempotencyConflict
	}
	return json.Unmarshal(stored.ResponseSnapshot, response)
}

func replayReplan(actorID uuid.UUID, requestHash string, stored storedBatch, response *api.ScheduleReplanResponse) error {
	if stored.ActorType != "human" || stored.RequestedBy == nil || *stored.RequestedBy != actorID {
		return api.ErrScheduleForbidden
	}
	if strings.TrimSpace(stored.RequestHash) != requestHash {
		return ErrScheduleIdempotencyConflict
	}
	return json.Unmarshal(stored.ResponseSnapshot, response)
}

type lockedPreview struct {
	ID             uuid.UUID
	RequestHash    string
	ResultHash     string
	RequestedBy    *uuid.UUID
	ActorType      string
	ActorName      *string
	ExpiresAt      time.Time
	ResultSnapshot []byte
}

func lockPreview(ctx context.Context, tx DBTX, actorID, id uuid.UUID, operation, resultHash string) (lockedPreview, error) {
	var preview lockedPreview
	err := tx.QueryRow(ctx, `SELECT id, request_hash, result_hash, requested_by, actor_type, actor_name, expires_at, result_snapshot FROM schedule_previews WHERE id = $1 FOR UPDATE`, id).Scan(&preview.ID, &preview.RequestHash, &preview.ResultHash, &preview.RequestedBy, &preview.ActorType, &preview.ActorName, &preview.ExpiresAt, &preview.ResultSnapshot)
	if errors.Is(err, pgx.ErrNoRows) {
		return lockedPreview{}, ErrSchedulePreviewStale
	}
	if err != nil {
		return lockedPreview{}, err
	}
	if preview.ActorType != "human" || preview.RequestedBy == nil || *preview.RequestedBy != actorID || preview.ResultHash != resultHash || preview.ExpiresAt.Before(time.Now().UTC()) {
		return lockedPreview{}, ErrSchedulePreviewStale
	}
	var storedOperation struct {
		Operation string `json:"operation"`
	}
	if err := json.Unmarshal(preview.ResultSnapshot, &storedOperation); err == nil && storedOperation.Operation != "" && storedOperation.Operation != operation {
		return lockedPreview{}, ErrSchedulePreviewStale
	}
	return preview, nil
}

func loadCurrentRevision(ctx context.Context, tx DBTX, employmentID uuid.UUID, businessDate string, forUpdate bool) (lifecycleRevision, error) {
	var revision lifecycleRevision
	err := tx.QueryRow(ctx, `SELECT r.id, r.employment_id, r.employer_id, r.business_date::text, r.timezone, r.revision_number, r.state, r.publication_batch_id, r.source, r.source_snapshot, r.published_at, b.mode, b.approval_instance_id FROM schedule_day_revisions r LEFT JOIN schedule_publication_batches b ON b.id = r.publication_batch_id WHERE r.employment_id = $1 AND r.business_date = $2::date AND r.state IN ('published', 'explicit_off') ORDER BY r.revision_number DESC LIMIT 1`, employmentID, businessDate).Scan(&revision.ID, &revision.EmploymentID, &revision.EmployerID, &revision.BusinessDate, &revision.Timezone, &revision.RevisionNumber, &revision.State, &revision.BatchID, &revision.Source, &revision.SourceSnapshot, &revision.PublishedAt, &revision.Mode, &revision.ApprovalInstanceID)
	if errors.Is(err, pgx.ErrNoRows) {
		return lifecycleRevision{}, ErrScheduleNoCurrentRevision
	}
	if err != nil {
		return lifecycleRevision{}, err
	}
	if forUpdate {
		if _, err := tx.Exec(ctx, `SELECT id FROM schedule_day_revisions WHERE id = $1 FOR UPDATE`, revision.ID); err != nil {
			return lifecycleRevision{}, err
		}
	}
	return revision, nil
}

func ensureMutationPeriodOpen(ctx context.Context, tx DBTX, employerID uuid.UUID, businessDate string) error {
	var payrollLocked, attendanceClosed bool
	err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_periods WHERE employer_id = $1 AND status = 'locked' AND make_date(period_year, period_month, 1) <= $2::date AND (make_date(period_year, period_month, 1) + interval '1 month - 1 day') >= $2::date), EXISTS (SELECT 1 FROM attendance_periods WHERE employer_id = $1 AND status = 'closed' AND make_date(period_year, period_month, 1) <= $2::date AND (make_date(period_year, period_month, 1) + interval '1 month - 1 day') >= $2::date)`, employerID, businessDate).Scan(&payrollLocked, &attendanceClosed)
	if err != nil {
		return err
	}
	if payrollLocked || attendanceClosed {
		return ErrScheduleLockedPeriod
	}
	return nil
}

func ensureMutationPeriodOpenForRange(ctx context.Context, tx DBTX, employmentIDs []uuid.UUID, startDate, endDate string) error {
	var blocked bool
	err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_periods pp JOIN employments e ON e.employer_id = pp.employer_id WHERE e.id = ANY($1) AND pp.status = 'locked' AND make_date(pp.period_year, pp.period_month, 1) <= $3::date AND (make_date(pp.period_year, pp.period_month, 1) + interval '1 month - 1 day') >= $2::date) OR EXISTS (SELECT 1 FROM attendance_periods ap JOIN employments e ON e.employer_id = ap.employer_id WHERE e.id = ANY($1) AND ap.status = 'closed' AND make_date(ap.period_year, ap.period_month, 1) <= $3::date AND (make_date(ap.period_year, ap.period_month, 1) + interval '1 month - 1 day') >= $2::date)`, employmentIDs, startDate, endDate).Scan(&blocked)
	if err != nil {
		return err
	}
	if blocked {
		return ErrScheduleLockedPeriod
	}
	return nil
}

func verifyFirstPublicationTargets(ctx context.Context, tx DBTX, request api.SchedulePublishRequest) error {
	var current bool
	err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM schedule_day_revisions WHERE employment_id = ANY($1) AND business_date BETWEEN $2::date AND $3::date AND state IN ('published', 'explicit_off'))`, request.EmploymentIDs, request.StartDate, request.EndDate).Scan(&current)
	if err != nil {
		return err
	}
	if current {
		return ErrScheduleAlreadyPublished
	}
	return nil
}

type publicationActor struct {
	ID   *uuid.UUID
	Type string
	Name *string
}

type scheduleRevisionLockKey struct {
	employmentID uuid.UUID
	businessDate string
}

func lockScheduleRevisionTargets(ctx context.Context, tx DBTX, keys []scheduleRevisionLockKey) error {
	if len(keys) == 0 {
		return nil
	}
	unique := make(map[string]scheduleRevisionLockKey, len(keys))
	for _, key := range keys {
		if key.employmentID == uuid.Nil || key.businessDate == "" {
			return ErrSchedulePreviewScope
		}
		unique[key.employmentID.String()+":"+key.businessDate] = key
	}
	keys = keys[:0]
	for _, key := range unique {
		keys = append(keys, key)
	}
	sort.Slice(keys, func(i, j int) bool {
		left := keys[i].employmentID.String() + ":" + keys[i].businessDate
		right := keys[j].employmentID.String() + ":" + keys[j].businessDate
		return left < right
	})
	employmentIDs := make([]uuid.UUID, 0, len(keys))
	dates := make([]string, 0, len(keys))
	seenEmployments := make(map[uuid.UUID]struct{}, len(keys))
	for _, key := range keys {
		dates = append(dates, key.businessDate)
		if _, exists := seenEmployments[key.employmentID]; !exists {
			employmentIDs = append(employmentIDs, key.employmentID)
			seenEmployments[key.employmentID] = struct{}{}
		}
	}
	sort.Strings(dates)
	sort.Slice(employmentIDs, func(i, j int) bool { return employmentIDs[i].String() < employmentIDs[j].String() })
	if _, err := tx.Exec(ctx, `SELECT e.id FROM employees e JOIN employments em ON em.employee_id = e.id WHERE em.id = ANY($1) ORDER BY e.id FOR UPDATE`, employmentIDs); err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `SELECT id FROM employments WHERE id = ANY($1) ORDER BY id FOR UPDATE`, employmentIDs); err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `SELECT id FROM schedule_day_revisions WHERE employment_id = ANY($1) AND business_date BETWEEN $2::date AND $3::date ORDER BY id FOR UPDATE`, employmentIDs, dates[0], dates[len(dates)-1]); err != nil {
		return err
	}
	return nil
}

func hashMap(value any) string {
	hash, err := hashJSON(value)
	if err != nil {
		return ""
	}
	return hash
}

func stringPtr(value string) *string {
	if value == "" {
		return nil
	}
	return &value
}

func publishHashInput(actorID uuid.UUID, request api.SchedulePublishRequest, previewID, previewHash string) lifecycleRequestHashInput {
	approval := ""
	if request.ApprovalInstanceID != nil {
		approval = request.ApprovalInstanceID.String()
	}
	mode := request.Mode
	if mode == "" {
		mode = "office"
	}
	return lifecycleRequestHashInput{Operation: "publish", ActorID: actorID.String(), EmploymentIDs: uuidStrings(request.EmploymentIDs), StartDate: request.StartDate, EndDate: request.EndDate, Timezone: request.Timezone, Mode: mode, ApprovalInstanceID: stringPtr(approval), ExpectedRevisions: expectedRevisionList(request.ExpectedRevisions), Reason: strings.TrimSpace(request.Reason), Retroactive: request.Retroactive, PreviewID: previewID, PreviewHash: previewHash}
}

func replanHashInput(actorID, employmentID uuid.UUID, businessDate string, request api.ScheduleReplanPreviewRequest, previewID, previewHash, confirmDiffHash string) lifecycleRequestHashInput {
	return lifecycleRequestHashInput{Operation: "replan", ActorID: actorID.String(), EmploymentID: employmentID.String(), BusinessDate: businessDate, ExpectedRevision: request.ExpectedRevision, Reason: strings.TrimSpace(request.Reason), Replacement: &request.Replacement, PreviewID: previewID, PreviewHash: previewHash, ConfirmDiffHash: confirmDiffHash}
}

func expectedRevisionList(values map[string]int64) []lifecycleExpectedRevision {
	result := make([]lifecycleExpectedRevision, 0, len(values))
	for key, value := range values {
		result = append(result, lifecycleExpectedRevision{Key: strings.ToLower(strings.TrimSpace(key)), Value: value})
	}
	sort.Slice(result, func(i, j int) bool { return result[i].Key < result[j].Key })
	return result
}

func uuidStrings(values []uuid.UUID) []string {
	result := make([]string, len(values))
	for i, value := range values {
		result[i] = value.String()
	}
	sort.Strings(result)
	return result
}

func hashJSON(value any) (string, error) {
	payload, err := json.Marshal(value)
	if err != nil {
		return "", err
	}
	digest := sha256.Sum256(payload)
	return hex.EncodeToString(digest[:]), nil
}

func jsonArg(value any) []byte {
	payload, _ := json.Marshal(value)
	return payload
}

func validatePublishInput(key string, request api.SchedulePublishRequest) error {
	mode := request.Mode
	if mode == "" {
		mode = "office"
	}
	if strings.TrimSpace(key) == "" || len(key) > 128 || !isASCIIKey(key) || (mode != "office" && mode != "rotating") || (mode == "office" && request.ApprovalInstanceID == nil) || (mode == "rotating" && request.ApprovalInstanceID != nil) || request.PreviewID == uuid.Nil || !isHash(request.PreviewHash) || strings.TrimSpace(request.Reason) == "" || len(request.EmploymentIDs) == 0 {
		return api.ErrScheduleValidation
	}
	if _, err := time.Parse("2006-01-02", request.StartDate); err != nil {
		return api.ErrScheduleValidation
	}
	if _, err := time.Parse("2006-01-02", request.EndDate); err != nil {
		return api.ErrScheduleValidation
	}
	if _, err := time.LoadLocation(request.Timezone); err != nil {
		return api.ErrScheduleValidation
	}
	return nil
}

func validateReplanInput(employmentID uuid.UUID, businessDate string, request api.ScheduleReplanPreviewRequest) error {
	if employmentID == uuid.Nil || request.ExpectedRevision <= 0 || len(strings.TrimSpace(request.Reason)) == 0 || len(strings.TrimSpace(request.Reason)) > 2000 || !validReplacementShape(request.Replacement) {
		return api.ErrScheduleValidation
	}
	if _, err := time.Parse("2006-01-02", businessDate); err != nil {
		return api.ErrScheduleValidation
	}
	return nil
}

func validateCommitReplanInput(employmentID uuid.UUID, businessDate, key string, request api.ScheduleReplanRequest) error {
	if err := validateReplanInput(employmentID, businessDate, api.ScheduleReplanPreviewRequest{ExpectedRevision: request.ExpectedRevision, Reason: request.Reason, Replacement: request.Replacement}); err != nil {
		return err
	}
	if strings.TrimSpace(key) == "" || len(key) > 128 || !isASCIIKey(key) || request.PreviewID == uuid.Nil || !isHash(request.PreviewHash) || !isHash(request.ConfirmDiffHash) {
		return api.ErrScheduleValidation
	}
	return nil
}

func validReplacementShape(value api.ScheduleReplanReplacement) bool {
	return value.Status == "work" && value.ShiftVersionID != nil && value.Source != "" && len(value.Intervals) > 0
}

func isASCIIKey(value string) bool {
	for i := range value {
		if value[i] < 0x20 || value[i] > 0x7e {
			return false
		}
	}
	return true
}

func isHash(value string) bool {
	if len(value) != 64 {
		return false
	}
	_, err := hex.DecodeString(value)
	return err == nil && strings.ToLower(value) == value
}

func uuidFromMap(value map[string]any, key string) (uuid.UUID, error) {
	text, ok := value[key].(string)
	if !ok {
		return uuid.Nil, errors.New("schedule lifecycle preview has invalid UUID")
	}
	return uuid.Parse(text)
}

func parseLocalTime(value string) (time.Time, error) {
	for _, layout := range []string{"15:04", "15:04:05"} {
		if parsed, err := time.Parse(layout, value); err == nil {
			return parsed, nil
		}
	}
	return time.Time{}, fmt.Errorf("invalid local time %q", value)
}

func parseDateParts(value string) (int, time.Month, int) {
	parsed, _ := time.Parse("2006-01-02", value)
	return parsed.Year(), parsed.Month(), parsed.Day()
}

func dayIntervals(day map[string]any, timezone string, businessDate string) ([]normalizedInterval, error) {
	payload, ok := day["intervals"]
	if !ok {
		return nil, errors.New("schedule lifecycle preview has no intervals")
	}
	encoded, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}
	var intervals []struct {
		SortOrder             int              `json:"sort_order"`
		Start                 string           `json:"start"`
		End                   string           `json:"end"`
		StartAt               time.Time        `json:"start_at"`
		EndAt                 time.Time        `json:"end_at"`
		ShiftVersionID        string           `json:"shift_version_id"`
		Breaks                []map[string]any `json:"breaks"`
		BreakPunchMode        string           `json:"break_punch_mode"`
		StartDayOffset        int              `json:"start_day_offset"`
		EndDayOffset          int              `json:"end_day_offset"`
		StartUTCOffsetMinutes *int             `json:"start_utc_offset_minutes"`
		EndUTCOffsetMinutes   *int             `json:"end_utc_offset_minutes"`
	}
	if err := json.Unmarshal(encoded, &intervals); err != nil {
		return nil, err
	}
	result := make([]normalizedInterval, 0, len(intervals))
	for _, item := range intervals {
		versionID, err := uuid.Parse(item.ShiftVersionID)
		if err != nil {
			return nil, err
		}
		result = append(result, normalizedInterval{SortOrder: item.SortOrder, StartLocal: item.Start, EndLocal: item.End, StartAt: item.StartAt, EndAt: item.EndAt, ShiftVersionID: versionID, Breaks: item.Breaks, BreakPunchMode: item.BreakPunchMode, StartDayOffset: item.StartDayOffset, EndDayOffset: item.EndDayOffset, StartUTCOffsetMinutes: item.StartUTCOffsetMinutes, EndUTCOffsetMinutes: item.EndUTCOffsetMinutes})
	}
	if len(result) == 0 {
		return nil, errors.New("schedule lifecycle preview has no work intervals")
	}
	_ = timezone
	_ = businessDate
	return result, nil
}
