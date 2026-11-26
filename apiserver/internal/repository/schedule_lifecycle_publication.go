package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
	"strings"
	"time"
)

func (r ScheduleRepository) publishHumanTx(ctx context.Context, tx pgx.Tx, actorID uuid.UUID, idempotencyKey string, request api.SchedulePublishRequest, response *api.SchedulePublicationResponse) error {
	if err := lockIdempotencyKey(ctx, tx, idempotencyKey); err != nil {
		return err
	}
	requestHash, err := hashJSON(publishHashInput(actorID, request, "", ""))
	if err != nil {
		return err
	}
	stored, found, err := loadBatchByIdempotency(ctx, tx, idempotencyKey)
	if err != nil {
		return err
	}
	if found {
		return replayPublication(actorID, requestHash, stored, response)
	}
	preview, err := lockPreview(ctx, tx, actorID, request.PreviewID, "publish", request.PreviewHash)
	if err != nil {
		return err
	}
	if preview.RequestHash != requestHash {
		return ErrSchedulePreviewStale
	}
	if err := validateHumanPublishPreconditions(ctx, tx, request); err != nil {
		return err
	}
	return publishPreview(ctx, tx, publicationActor{ID: &actorID, Type: "human"}, idempotencyKey, requestHash, request, preview, response)
}

func validateHumanPublishPreconditions(ctx context.Context, tx pgx.Tx, request api.SchedulePublishRequest) error {
	if err := ensureMutationPeriodOpenForRange(ctx, tx, request.EmploymentIDs, request.StartDate, request.EndDate); err != nil {
		return err
	}
	if request.Mode != "rotating" {
		if err := validateHumanOfficeApproval(ctx, tx, request); err != nil {
			return err
		}
	}
	return verifyFirstPublicationTargets(ctx, tx, request)
}

func (r ScheduleRepository) Publish(ctx context.Context, actorID uuid.UUID, idempotencyKey string, request api.SchedulePublishRequest) (api.SchedulePublicationResponse, error) {
	if err := validatePublishInput(idempotencyKey, request); err != nil {
		return api.SchedulePublicationResponse{}, err
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return api.SchedulePublicationResponse{}, err
	}
	var response api.SchedulePublicationResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		return r.publishHumanTx(ctx, tx, actorID, idempotencyKey, request, &response)
	})
	if err != nil {
		return api.SchedulePublicationResponse{}, err
	}
	return response, nil
}

// PublishSystemScheduleOffice publishes one approved future employment/day using the same publication transaction as human office publication.
func validateSystemOfficePublicationAuthority(ctx context.Context, tx DBTX, event ScheduleOfficeCoverageEvent) error {
	var allowed bool
	if err := tx.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1
		FROM schedule_office_policies p
		JOIN approval_workflow_instances i ON i.id = p.approval_instance_id
		JOIN employments e ON e.id = $2 AND e.employer_id = p.employer_id AND e.deleted_at IS NULL
		WHERE p.approval_instance_id = $1 AND p.employer_id = $3
		  AND p.status = 'active' AND i.entity_type = 'schedule_policy' AND i.status = 'approved'
		  AND p.effective_from <= $4::date AND (p.effective_to IS NULL OR p.effective_to >= $4::date)
		  AND e.start_date <= $4::date AND (e.end_date IS NULL OR e.end_date >= $4::date)
	)`, event.ApprovalInstanceID, event.EmploymentID, event.EmployerID, event.BusinessDate).Scan(&allowed); err != nil {
		return err
	}
	if !allowed {
		return ErrScheduleOfficePolicyUnavailable
	}
	return nil
}

func resolveSystemOfficeDay(ctx context.Context, tx pgx.Tx, event ScheduleOfficeCoverageEvent, request *api.SchedulePublishRequest) (map[string]any, error) {
	resolution, _, err := resolveScheduleData(ctx, tx, parseScheduleDate(event.BusinessDate), parseScheduleDate(event.BusinessDate), request.EmploymentIDs, 1, 1)
	if err != nil {
		return nil, err
	}
	if len(resolution.employees) != 1 || len(resolution.employees[0].days) != 1 {
		return nil, ErrScheduleResolverUnavailable
	}
	request.Timezone = resolution.employees[0].days[0].timezone
	day, err := sanitizedPreviewDay(resolution.employees[0].days[0], resolution.shifts)
	if err != nil {
		return nil, err
	}
	day["employment_id"] = event.EmploymentID.String()
	if day["status"] != "work" && day["status"] != "explicit_off" {
		return nil, ErrScheduleResolverUnavailable
	}
	return day, nil
}

func insertSystemOfficePreview(ctx context.Context, tx pgx.Tx, event ScheduleOfficeCoverageEvent, idempotencyKey string, request api.SchedulePublishRequest, day map[string]any) (lockedPreview, string, error) {
	result := map[string]any{"operation": "publish", "data": []map[string]any{day}, "resolver_version": lifecycleResolverVersion, "expected_revisions": expectedRevisionList(request.ExpectedRevisions)}
	resultHash, err := hashJSON(result)
	if err != nil {
		return lockedPreview{}, "", err
	}
	requestHash, err := hashJSON(map[string]any{"operation": "publish", "actor_type": "system", "actor_name": "schedule-publisher", "job_id": idempotencyKey, "approval_instance_id": event.ApprovalInstanceID.String(), "employment_id": event.EmploymentID.String(), "business_date": event.BusinessDate})
	if err != nil {
		return lockedPreview{}, "", err
	}
	createdAt := time.Now().UTC()
	previewID := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_previews (id, requested_by, actor_type, actor_name, operation, request_hash, result_hash, scope_snapshot, result_snapshot, resolver_version, expires_at, created_at) VALUES ($1, NULL, 'system', 'schedule-publisher', 'publish', $2, $3, $4, $5, $6, $7, $8)`, previewID, requestHash, resultHash, jsonArg(map[string]any{"employment_ids": request.EmploymentIDs, "start_date": event.BusinessDate, "end_date": event.BusinessDate, "approval_instance_id": event.ApprovalInstanceID}), jsonArg(result), lifecycleResolverVersion, createdAt.Add(previewTTL), createdAt); err != nil {
		return lockedPreview{}, "", err
	}
	return lockedPreview{ID: previewID, RequestHash: requestHash, ResultHash: resultHash, ActorType: "system", ActorName: stringPtr("schedule-publisher"), ExpiresAt: createdAt.Add(previewTTL), ResultSnapshot: jsonArg(result)}, requestHash, nil
}

func checkSystemOfficeIdempotency(ctx context.Context, tx pgx.Tx, idempotencyKey string) (uuid.UUID, bool, error) {
	if err := lockIdempotencyKey(ctx, tx, idempotencyKey); err != nil {
		return uuid.Nil, false, err
	}
	stored, found, err := loadBatchByIdempotency(ctx, tx, idempotencyKey)
	if err != nil || !found {
		return uuid.Nil, false, err
	}
	if stored.ActorType != "system" || stored.ActorName == nil || *stored.ActorName != "schedule-publisher" {
		return uuid.Nil, false, ErrScheduleIdempotencyConflict
	}
	return stored.ID, true, nil
}

func validateSystemOfficePreconditions(ctx context.Context, tx pgx.Tx, event ScheduleOfficeCoverageEvent, request api.SchedulePublishRequest) (bool, error) {
	if err := validateSystemOfficePublicationAuthority(ctx, tx, event); err != nil {
		return false, err
	}
	if err := ensureMutationPeriodOpen(ctx, tx, event.EmployerID, event.BusinessDate); err != nil {
		return false, err
	}
	if err := verifyFirstPublicationTargets(ctx, tx, request); err != nil {
		if errors.Is(err, ErrScheduleAlreadyPublished) {
			return true, nil
		}
		return false, err
	}
	return false, nil
}

func publishSystemOfficeTx(ctx context.Context, tx pgx.Tx, event ScheduleOfficeCoverageEvent, idempotencyKey string) (uuid.UUID, error) {
	if existingID, found, err := checkSystemOfficeIdempotency(ctx, tx, idempotencyKey); err != nil || found {
		return existingID, err
	}
	request := api.SchedulePublishRequest{
		EmploymentIDs:      []uuid.UUID{event.EmploymentID},
		StartDate:          event.BusinessDate,
		EndDate:            event.BusinessDate,
		ApprovalInstanceID: &event.ApprovalInstanceID,
		Reason:             "Approved office policy publication",
		ExpectedRevisions:  map[string]int64{event.EmploymentID.String() + ":" + event.BusinessDate: 0},
	}
	if alreadyPublished, err := validateSystemOfficePreconditions(ctx, tx, event, request); err != nil || alreadyPublished {
		return uuid.Nil, err
	}
	day, err := resolveSystemOfficeDay(ctx, tx, event, &request)
	if err != nil {
		return uuid.Nil, err
	}
	preview, requestHash, err := insertSystemOfficePreview(ctx, tx, event, idempotencyKey, request, day)
	if err != nil {
		return uuid.Nil, err
	}
	actorName := "schedule-publisher"
	if err := publishPreview(ctx, tx, publicationActor{Type: "system", Name: &actorName}, idempotencyKey, requestHash, request, preview, nil); err != nil {
		return uuid.Nil, err
	}
	var batchID uuid.UUID
	err = tx.QueryRow(ctx, `SELECT id FROM schedule_publication_batches WHERE idempotency_key = $1`, idempotencyKey).Scan(&batchID)
	return batchID, err
}

func (r ScheduleRepository) PublishSystemScheduleOffice(ctx context.Context, payload []byte, idempotencyKey string) (string, error) {
	var event ScheduleOfficeCoverageEvent
	if err := json.Unmarshal(payload, &event); err != nil {
		return "", fmt.Errorf("decode schedule office event: %w", err)
	}
	if err := validateScheduleOfficeEvent(event); err != nil {
		return "", err
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return "", err
	}
	var batchID uuid.UUID
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		id, err := publishSystemOfficeTx(ctx, tx, event, idempotencyKey)
		if err != nil {
			return err
		}
		batchID = id
		return nil
	})
	if err != nil {
		return "", err
	}
	return batchID.String(), nil
}

func rejectConcreteScheduleOverlaps(ctx context.Context, tx DBTX, days []map[string]any) error {
	type candidate struct {
		EmploymentID string    `json:"employment_id"`
		BusinessDate string    `json:"business_date"`
		StartAt      time.Time `json:"start_at"`
		EndAt        time.Time `json:"end_at"`
	}
	candidates := make([]candidate, 0)
	for _, day := range days {
		if status, _ := day["status"].(string); status != "work" {
			continue
		}
		employmentID, err := uuidFromMap(day, "employment_id")
		if err != nil {
			return err
		}
		intervals, err := dayIntervals(day, "UTC", "")
		if err != nil {
			return err
		}
		businessDate, _ := day["business_date"].(string)
		for _, interval := range intervals {
			candidates = append(candidates, candidate{EmploymentID: employmentID.String(), BusinessDate: businessDate, StartAt: interval.StartAt, EndAt: interval.EndAt})
		}
	}
	if len(candidates) == 0 {
		return nil
	}
	var conflict bool
	err := tx.QueryRow(ctx, `
		WITH candidates AS (
			SELECT * FROM jsonb_to_recordset($1::jsonb) AS c(employment_id uuid, business_date date, start_at timestamptz, end_at timestamptz)
		), candidate_people AS (
			SELECT c.*, em.employee_id
			FROM candidates c JOIN employments em ON em.id = c.employment_id
		), existing AS (
			SELECT r.business_date, ws.scheduled_start_at AS start_at, ws.scheduled_end_at AS end_at, em.employee_id, ws.employment_id
			FROM work_segments ws
			JOIN schedule_day_revisions r ON r.id = ws.schedule_revision_id AND r.state IN ('published', 'explicit_off')
			JOIN employments em ON em.id = ws.employment_id
		)
		SELECT EXISTS (
			SELECT 1 FROM candidate_people c JOIN existing e ON e.employee_id = c.employee_id
			WHERE (e.employment_id <> c.employment_id OR e.business_date <> c.business_date)
			  AND c.start_at < e.end_at AND e.start_at < c.end_at
			UNION ALL
			SELECT 1 FROM candidate_people a JOIN candidate_people b ON a.employee_id = b.employee_id
			WHERE (a.employment_id, a.business_date, a.start_at) < (b.employment_id, b.business_date, b.start_at)
			  AND a.start_at < b.end_at AND b.start_at < a.end_at
		)`, jsonArg(candidates)).Scan(&conflict)
	if err != nil {
		return err
	}
	if conflict {
		return api.ErrScheduleConflict
	}
	return nil
}

func extractPreviewLockKeys(days []map[string]any) ([]scheduleRevisionLockKey, error) {
	keys := make([]scheduleRevisionLockKey, 0, len(days))
	for _, day := range days {
		employmentID, err := uuidFromMap(day, "employment_id")
		if err != nil {
			return nil, err
		}
		businessDate, ok := day["business_date"].(string)
		if !ok || businessDate == "" {
			return nil, ErrSchedulePreviewScope
		}
		keys = append(keys, scheduleRevisionLockKey{employmentID: employmentID, businessDate: businessDate})
	}
	return keys, nil
}

func finalizePublicationBatch(ctx context.Context, tx pgx.Tx, batchID uuid.UUID, actor publicationActor, publishedAt time.Time, revisionIDs []uuid.UUID, response *api.SchedulePublicationResponse) error {
	publicationResponse := api.SchedulePublicationResponse{BatchID: batchID, Status: "published", PublishedAt: publishedAt, RevisionIDs: revisionIDs, AffectedDays: len(revisionIDs)}
	if response != nil {
		*response = publicationResponse
	}
	responseJSON, err := json.Marshal(publicationResponse)
	if err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `UPDATE schedule_publication_batches SET response_snapshot = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $1`, batchID, responseJSON); err != nil {
		return err
	}
	_, err = tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, created_at, actor_type, actor_name) VALUES ($1, $2, 'schedule.published', $3, CURRENT_TIMESTAMP, $4, $5)`, actor.ID, batchID, jsonArg(map[string]any{"batch_id": batchID.String(), "revision_ids": revisionIDs, "affected_days": len(revisionIDs)}), actor.Type, actor.Name)
	return err
}

func preparePublicationBatch(ctx context.Context, tx pgx.Tx, actor publicationActor, idempotencyKey, requestHash string, request api.SchedulePublishRequest, days []map[string]any) (uuid.UUID, time.Time, error) {
	keys, err := extractPreviewLockKeys(days)
	if err != nil {
		return uuid.Nil, time.Time{}, err
	}
	if err := lockScheduleRevisionTargets(ctx, tx, keys); err != nil {
		return uuid.Nil, time.Time{}, err
	}
	if err := rejectConcreteScheduleOverlaps(ctx, tx, days); err != nil {
		return uuid.Nil, time.Time{}, err
	}
	batchID := uuid.New()
	publishedAt := time.Now().UTC()
	mode := request.Mode
	if mode == "" {
		mode = "office"
	}
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, actor_type, actor_name, approval_instance_id, reason, timezone, range_start, range_end, published_at, affected_days) VALUES ($1, $2, $3, $4, 'published', $5, 'publish', '{}'::jsonb, $6, $7, $8, $9, $10, $11::date, $12::date, $13, $14)`, batchID, idempotencyKey, requestHash, mode, actor.ID, actor.Type, actor.Name, request.ApprovalInstanceID, strings.TrimSpace(request.Reason), request.Timezone, request.StartDate, request.EndDate, publishedAt, len(days)); err != nil {
		return uuid.Nil, time.Time{}, err
	}
	if _, err := tx.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
		return uuid.Nil, time.Time{}, err
	}
	return batchID, publishedAt, nil
}

func publishPreview(ctx context.Context, tx pgx.Tx, actor publicationActor, idempotencyKey, requestHash string, request api.SchedulePublishRequest, preview lockedPreview, response *api.SchedulePublicationResponse) error {
	var snapshot struct {
		Data []map[string]any `json:"data"`
	}
	if err := json.Unmarshal(preview.ResultSnapshot, &snapshot); err != nil || len(snapshot.Data) == 0 {
		return ErrScheduleResolverUnavailable
	}
	batchID, publishedAt, err := preparePublicationBatch(ctx, tx, actor, idempotencyKey, requestHash, request, snapshot.Data)
	if err != nil {
		return err
	}
	revisionIDs := make([]uuid.UUID, 0, len(snapshot.Data))
	for _, day := range snapshot.Data {
		revisionID, err := insertPublishedDay(ctx, tx, batchID, actor, request.Reason, request.Timezone, day, publishedAt, nil)
		if err != nil {
			return err
		}
		revisionIDs = append(revisionIDs, revisionID)
	}
	return finalizePublicationBatch(ctx, tx, batchID, actor, publishedAt, revisionIDs, response)
}

func parsePublishedDayMeta(day map[string]any) (uuid.UUID, string, string, string, error) {
	employmentID, err := uuidFromMap(day, "employment_id")
	if err != nil {
		return uuid.Nil, "", "", "", err
	}
	businessDate, ok := day["business_date"].(string)
	if !ok || businessDate == "" {
		return uuid.Nil, "", "", "", errors.New("schedule lifecycle preview has invalid business date")
	}
	status, _ := day["status"].(string)
	source, _ := day["source"].(string)
	if status != "work" && status != "explicit_off" {
		return uuid.Nil, "", "", "", ErrScheduleResolverUnavailable
	}
	if source == "" {
		return uuid.Nil, "", "", "", errors.New("schedule lifecycle preview has no source")
	}
	return employmentID, businessDate, status, source, nil
}

func insertPublishedWorkSegments(ctx context.Context, tx pgx.Tx, employmentID, revisionID uuid.UUID, businessDate, source string, intervals []normalizedInterval) error {
	groupKey := fmt.Sprintf("%s:%s", employmentID, businessDate)
	for _, interval := range intervals {
		if _, err := tx.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, template_id, interval_id, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, source, source_detail, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot, start_day_offset, end_day_offset, shift_group_key, start_utc_offset_minutes, end_utc_offset_minutes) SELECT $1, $2, $3::date, sv.template_id, NULL, $4, $5::time, $6::time, 'work', 'scheduled', $7, '{}'::jsonb, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17 FROM shift_versions sv WHERE sv.id = $9`, uuid.New(), employmentID, businessDate, interval.SortOrder, interval.StartLocal, interval.EndLocal, source, revisionID, interval.ShiftVersionID, interval.StartAt, interval.EndAt, jsonArg(interval.Breaks), interval.StartDayOffset, interval.EndDayOffset, groupKey, interval.StartUTCOffsetMinutes, interval.EndUTCOffsetMinutes); err != nil {
			return err
		}
	}
	return nil
}

func insertPublishedDay(ctx context.Context, tx pgx.Tx, batchID uuid.UUID, actor publicationActor, reason, timezone string, day map[string]any, publishedAt time.Time, previousRevisionID *uuid.UUID) (uuid.UUID, error) {
	employmentID, businessDate, status, source, err := parsePublishedDayMeta(day)
	if err != nil {
		return uuid.Nil, err
	}
	revisionID := uuid.New()
	var revisionNumber int64
	if err := tx.QueryRow(ctx, `SELECT COALESCE(MAX(revision_number), 0) + 1 FROM schedule_day_revisions WHERE employment_id = $1 AND business_date = $2::date`, employmentID, businessDate).Scan(&revisionNumber); err != nil {
		return uuid.Nil, err
	}
	sourceSnapshot := jsonArg(map[string]any{"source": source, "resolver_version": lifecycleResolverVersion, "preview_result": day})
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_actor_name, published_at, previous_revision_id) SELECT $1, e.id, e.employer_id, $2::date, $3, $4, 'draft', $5, $6, $7, $8, $9, $10, $11, $12, NULL, $13 FROM employments e WHERE e.id = $14`, revisionID, businessDate, timezone, revisionNumber, batchID, source, sourceSnapshot, lifecycleResolverVersion, strings.TrimSpace(reason), actor.ID, actor.Type, actor.Name, previousRevisionID, employmentID); err != nil {
		return uuid.Nil, err
	}
	if status == "work" {
		intervals, err := dayIntervals(day, timezone, businessDate)
		if err != nil {
			return uuid.Nil, err
		}
		if err := insertPublishedWorkSegments(ctx, tx, employmentID, revisionID, businessDate, source, intervals); err != nil {
			return uuid.Nil, err
		}
	}
	state := "published"
	if status == "explicit_off" {
		state = "explicit_off"
	}
	if _, err := tx.Exec(ctx, `UPDATE schedule_day_revisions SET state = $2, published_at = $3 WHERE id = $1`, revisionID, state, publishedAt); err != nil {
		return uuid.Nil, err
	}
	return revisionID, nil
}
