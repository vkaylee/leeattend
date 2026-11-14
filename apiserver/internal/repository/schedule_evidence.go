package repository

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

func (r ScheduleRepository) GetScheduleActivationReadiness(ctx context.Context, employerID uuid.UUID, startDate, endDate string) (api.ScheduleActivationReadiness, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.ScheduleActivationReadiness{}, err
	}
	result := api.ScheduleActivationReadiness{EmployerID: employerID}
	if err := db.QueryRow(ctx, `SELECT COALESCE(enabled, false) FROM feature_flags WHERE flag_key = 'schedule_lifecycle'`).Scan(&result.GlobalFlagEnabled); err != nil && !errors.Is(err, pgx.ErrNoRows) {
		return result, err
	}
	if err := db.QueryRow(ctx, `SELECT COALESCE(bool_and(COALESCE(active.value_bool, false)), false)
		FROM generate_series($2::date, $3::date, interval '1 day') AS days(day)
		LEFT JOIN LATERAL (
			SELECT c.value_bool
			FROM config_entries c
			WHERE c.scope_type = 'employer'
			  AND c.scope_id = $1
			  AND c.config_key = 'schedule.lifecycle.enabled'
			  AND c.effective_from <= days.day::date
			  AND (c.effective_to IS NULL OR c.effective_to >= days.day::date)
			ORDER BY c.effective_from DESC, c.id DESC
			LIMIT 1
		) active ON true`, employerID, startDate, endDate).Scan(&result.EmployerActivation); err != nil {
		return result, err
	}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM schedule_shadow_mismatches WHERE employer_id = $1 AND business_date BETWEEN $2::date AND $3::date AND status = 'open'`, employerID, startDate, endDate).Scan(&result.OpenMismatchCount); err != nil {
		return result, err
	}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM (SELECT employment_id, business_date FROM schedule_day_revisions WHERE employer_id = $1 AND business_date BETWEEN $2::date AND $3::date AND state IN ('published', 'explicit_off') GROUP BY employment_id, business_date HAVING count(*) > 1) duplicates`, employerID, startDate, endDate).Scan(&result.CurrentDuplicateCount); err != nil {
		return result, err
	}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM schedule_legacy_mutation_evidence WHERE operation = 'guard_blocked' AND business_date BETWEEN $2::date AND $3::date AND employment_id IN (SELECT id FROM employments WHERE employer_id = $1)`, employerID, startDate, endDate).Scan(&result.LegacyMutationCount); err != nil {
		return result, err
	}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM outbox_jobs
		WHERE job_type = 'ScheduleOfficeCoverageRefresh'
		  AND status = 'dead_letter'
		  AND payload->>'employer_id' = $1
		  AND payload->>'business_date' BETWEEN $2 AND $3`, employerID.String(), startDate, endDate).Scan(&result.DeadLetterCount); err != nil {
		return result, err
	}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM employments e CROSS JOIN generate_series($2::date, $3::date, interval '1 day') d WHERE e.employer_id = $1 AND e.deleted_at IS NULL AND e.start_date <= d::date AND (e.end_date IS NULL OR e.end_date >= d::date)`, employerID, startDate, endDate).Scan(&result.ExpectedCoverageDays); err != nil {
		return result, err
	}
	if err := db.QueryRow(ctx, `SELECT count(DISTINCT r.employment_id::text || ':' || r.business_date::text) FROM schedule_day_revisions r WHERE r.employer_id = $1 AND r.business_date BETWEEN $2::date AND $3::date AND r.state IN ('published', 'explicit_off')`, employerID, startDate, endDate).Scan(&result.CoveredOfficialDays); err != nil {
		return result, err
	}
	result.MissingCoverageDays = result.ExpectedCoverageDays - result.CoveredOfficialDays
	if result.MissingCoverageDays < 0 {
		result.MissingCoverageDays = 0
	}
	result.Ready = result.GlobalFlagEnabled && result.EmployerActivation && result.OpenMismatchCount == 0 && result.CurrentDuplicateCount == 0 && result.DeadLetterCount == 0 && result.LegacyMutationCount == 0 && result.MissingCoverageDays == 0
	return result, nil
}

type ScheduleShadowMismatchRecord struct {
	EmployerID       uuid.UUID
	EmploymentID     uuid.UUID
	BusinessDate     string
	MismatchKind     string
	LegacySnapshot   json.RawMessage
	ResolverSnapshot json.RawMessage
	ResolverVersion  string
}

func (r ScheduleRepository) RecordScheduleMismatches(ctx context.Context, records []ScheduleShadowMismatchRecord) error {
	if len(records) == 0 {
		return nil
	}
	for _, record := range records {
		if err := validateScheduleShadowMismatchRecord(record); err != nil {
			return err
		}
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		placeholders := make([]string, len(records))
		args := make([]any, 0, len(records)*7)
		for i, record := range records {
			base := i*7 + 1
			placeholders[i] = fmt.Sprintf("($%d, $%d, $%d::date, $%d, $%d, $%d, $%d)", base, base+1, base+2, base+3, base+4, base+5, base+6)
			args = append(args, scheduleShadowMismatchArgs(record)...)
		}
		_, err := tx.Exec(ctx, `INSERT INTO schedule_shadow_mismatches
			(employer_id, employment_id, business_date, mismatch_kind, legacy_snapshot, resolver_snapshot, resolver_version)
			VALUES `+strings.Join(placeholders, ", ")+`
			ON CONFLICT (employer_id, employment_id, business_date, mismatch_kind) WHERE status = 'open'
			DO UPDATE SET legacy_snapshot = EXCLUDED.legacy_snapshot,
			              resolver_snapshot = EXCLUDED.resolver_snapshot,
			              resolver_version = EXCLUDED.resolver_version,
			              last_seen_at = CURRENT_TIMESTAMP`, args...)
		return err
	})
}

func validateScheduleShadowMismatchRecord(record ScheduleShadowMismatchRecord) error {
	if record.EmployerID == uuid.Nil || record.EmploymentID == uuid.Nil || record.ResolverVersion == "" {
		return fmt.Errorf("shadow mismatch identity is required")
	}
	if _, err := time.Parse("2006-01-02", record.BusinessDate); err != nil {
		return fmt.Errorf("shadow mismatch business date: %w", err)
	}
	validKinds := map[string]bool{"intervals": true, "overnight": true, "explicit_off": true, "missing": true, "effective_version": true, "timezone": true, "source": true, "other": true}
	if !validKinds[record.MismatchKind] {
		return fmt.Errorf("shadow mismatch kind is invalid")
	}
	for name, snapshot := range map[string]json.RawMessage{"legacy": record.LegacySnapshot, "resolver": record.ResolverSnapshot} {
		var object map[string]any
		if len(snapshot) == 0 || json.Unmarshal(snapshot, &object) != nil || object == nil {
			return fmt.Errorf("%s shadow snapshot must be a JSON object", name)
		}
	}
	return nil
}

func scheduleShadowMismatchArgs(record ScheduleShadowMismatchRecord) []any {
	return []any{record.EmployerID, record.EmploymentID, record.BusinessDate, record.MismatchKind, record.LegacySnapshot, record.ResolverSnapshot, record.ResolverVersion}
}

func (r ScheduleRepository) RecordScheduleShadowMismatch(ctx context.Context, record ScheduleShadowMismatchRecord) error {
	if err := validateScheduleShadowMismatchRecord(record); err != nil {
		return err
	}
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO schedule_shadow_mismatches
			(employer_id, employment_id, business_date, mismatch_kind, legacy_snapshot, resolver_snapshot, resolver_version)
			VALUES ($1, $2, $3::date, $4, $5, $6, $7)
			ON CONFLICT (employer_id, employment_id, business_date, mismatch_kind) WHERE status = 'open'
			DO UPDATE SET legacy_snapshot = EXCLUDED.legacy_snapshot,
			              resolver_snapshot = EXCLUDED.resolver_snapshot,
			              resolver_version = EXCLUDED.resolver_version,
			              last_seen_at = CURRENT_TIMESTAMP`, scheduleShadowMismatchArgs(record)...)
	return err
}

func (r ScheduleRepository) AuthorizeScheduleHistory(ctx context.Context, userID, employmentID uuid.UUID) (bool, error) {
	db, err := r.db(ctx)
	if err != nil {
		return false, err
	}
	var allowed bool
	err = db.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1 FROM employees e
		JOIN employments em ON em.employee_id = e.id
		WHERE em.id = $1 AND em.deleted_at IS NULL AND e.user_id = $2
		UNION ALL
		SELECT 1 FROM user_roles ur
		JOIN roles role ON role.id = ur.role_id
		JOIN role_permissions rp ON rp.role_id = role.id
		WHERE ur.user_id = $2 AND rp.permission_name = 'schedules:history:read' AND role.scope_type = 'global'
	)`, employmentID, userID).Scan(&allowed)
	return allowed, err
}

func (r ScheduleRepository) AuthorizeScheduleEmployment(ctx context.Context, userID, employmentID uuid.UUID, businessDate string) (bool, error) {
	db, err := r.db(ctx)
	if err != nil {
		return false, err
	}
	var allowed bool
	err = db.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1
		FROM employees e
		JOIN employments em ON em.employee_id = e.id
		WHERE em.id = $1 AND em.deleted_at IS NULL
		  AND em.start_date <= $2::date
		  AND (em.end_date IS NULL OR em.end_date >= $2::date)
		  AND (e.user_id = $3 OR EXISTS (
			SELECT 1 FROM user_roles ur
			JOIN roles role ON role.id = ur.role_id
			JOIN role_permissions rp ON rp.role_id = role.id
			JOIN employments manager_em ON manager_em.employee_id = (
				SELECT manager.id FROM employees manager WHERE manager.user_id = ur.user_id AND manager.deleted_at IS NULL LIMIT 1
			)
			WHERE ur.user_id = $3
			  AND rp.permission_name = 'schedules:evidence:export'
			  AND (role.scope_type = 'global' OR manager_em.employer_id = em.employer_id OR manager_em.department_id = em.department_id)
		  AND manager_em.deleted_at IS NULL
		  AND manager_em.start_date <= $2::date
		  AND (manager_em.end_date IS NULL OR manager_em.end_date >= $2::date)
		  ))
	)`, employmentID, businessDate, userID).Scan(&allowed)
	return allowed, err
}

func (r ScheduleRepository) ListScheduleHistory(ctx context.Context, query api.ScheduleHistoryQuery) (api.ScheduleHistoryResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.ScheduleHistoryResponse{}, err
	}
	args := []any{query.EmploymentID}
	where := []string{"employment_id = $1"}
	arg := 2
	if query.StartDate != nil {
		where = append(where, fmt.Sprintf("business_date >= $%d::date", arg))
		args = append(args, *query.StartDate)
		arg++
	}
	if query.EndDate != nil {
		where = append(where, fmt.Sprintf("business_date <= $%d::date", arg))
		args = append(args, *query.EndDate)
		arg++
	}
	if query.Cursor != nil {
		where = append(where, fmt.Sprintf(`
			(business_date, employment_id, revision_number, id) > ($%d::date, $%d::uuid, $%d::bigint, $%d::uuid)`, arg, arg+1, arg+2, arg+3))
		args = append(args, query.Cursor.BusinessDate, query.Cursor.EmploymentID, query.Cursor.RevisionNumber, query.Cursor.RevisionID)
		arg += 4
	}
	args = append(args, query.Limit+1)
	rows, err := db.Query(ctx, `SELECT id, employment_id, employer_id, business_date::text, timezone,
		revision_number, state, previous_revision_id, publication_batch_id, source, source_snapshot,
		resolver_version, reason, published_by, published_actor_type, published_actor_name, published_at, created_at
		FROM schedule_day_revisions
		WHERE `+strings.Join(where, " AND ")+`
		ORDER BY business_date ASC, employment_id ASC, revision_number ASC, id ASC
		LIMIT $`+fmt.Sprint(arg), args...)
	if err != nil {
		if isScheduleLifecycleSchemaError(err) {
			return api.ScheduleHistoryResponse{}, api.ErrScheduleEvidenceUnavailable
		}
		return api.ScheduleHistoryResponse{}, err
	}
	defer rows.Close()
	entries := make([]api.ScheduleHistoryEntry, 0, query.Limit)
	for rows.Next() {
		entry, err := scanScheduleHistoryEntry(rows)
		if err != nil {
			return api.ScheduleHistoryResponse{}, err
		}
		entries = append(entries, entry)
	}
	if err := rows.Err(); err != nil {
		if isScheduleLifecycleSchemaError(err) {
			return api.ScheduleHistoryResponse{}, api.ErrScheduleEvidenceUnavailable
		}
		return api.ScheduleHistoryResponse{}, err
	}
	var cursor *string
	if len(entries) > query.Limit {
		last := entries[query.Limit-1]
		entries = entries[:query.Limit]
		encoded, err := encodeHistoryCursor(api.ScheduleHistoryCursor{
			BusinessDate: last.BusinessDate, EmploymentID: last.EmploymentID,
			RevisionNumber: last.RevisionNumber, RevisionID: last.RevisionID,
		})
		if err != nil {
			return api.ScheduleHistoryResponse{}, err
		}
		cursor = &encoded
	}
	return api.ScheduleHistoryResponse{Data: entries, NextCursor: cursor}, nil
}

func (r ScheduleRepository) ResolveScheduleShadowMismatch(ctx context.Context, mismatchID uuid.UUID, status, note string, actorID uuid.UUID) error {
	if mismatchID == uuid.Nil || actorID == uuid.Nil || (status != "resolved" && status != "waived") || strings.TrimSpace(note) == "" || len(note) > 2000 {
		return fmt.Errorf("invalid shadow mismatch resolution")
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		note = strings.TrimSpace(note)
		result, err := tx.Exec(ctx, `UPDATE schedule_shadow_mismatches
			SET status = $2, resolution_note = $3, resolved_at = CURRENT_TIMESTAMP
			WHERE id = $1 AND status = 'open'`, mismatchID, status, note)
		if err != nil {
			return err
		}
		if result.RowsAffected() != 1 {
			return api.ErrScheduleShadowMismatchNotFound
		}
		_, err = tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, actor_type, created_at)
			VALUES ($1, $2, 'schedule.shadow_mismatch.resolved', jsonb_build_object('status', $3, 'note', $4), 'human', CURRENT_TIMESTAMP)`, actorID, mismatchID, status, note)
		return err
	})
}

func (r ScheduleRepository) ListScheduleShadowMismatches(ctx context.Context, employerID uuid.UUID, startDate, endDate string) ([]api.ScheduleShadowMismatch, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, employer_id, employment_id, business_date::text, mismatch_kind, legacy_snapshot, resolver_snapshot, resolver_version, status, first_seen_at, last_seen_at, resolved_at, resolution_note
		FROM schedule_shadow_mismatches
		WHERE employer_id = $1 AND business_date BETWEEN $2::date AND $3::date
		ORDER BY business_date ASC, employment_id ASC, id ASC`, employerID, startDate, endDate)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.ScheduleShadowMismatch, 0)
	for rows.Next() {
		var mismatch api.ScheduleShadowMismatch
		if err := rows.Scan(&mismatch.ID, &mismatch.EmployerID, &mismatch.EmploymentID, &mismatch.BusinessDate, &mismatch.MismatchKind, &mismatch.LegacySnapshot, &mismatch.ResolverSnapshot, &mismatch.ResolverVersion, &mismatch.Status, &mismatch.FirstSeenAt, &mismatch.LastSeenAt, &mismatch.ResolvedAt, &mismatch.ResolutionNote); err != nil {
			return nil, err
		}
		result = append(result, mismatch)
	}
	return result, rows.Err()
}

func (r ScheduleRepository) GetScheduleEvidence(ctx context.Context, actorID, employmentID uuid.UUID, businessDate string) (api.ScheduleEvidenceResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.ScheduleEvidenceResponse{}, err
	}
	revisions, err := r.loadScheduleEvidenceRevisions(ctx, db, employmentID, businessDate)
	if err != nil {
		return api.ScheduleEvidenceResponse{}, err
	}
	if len(revisions) == 0 {
		return api.ScheduleEvidenceResponse{}, api.ErrScheduleEvidenceNotFound
	}
	revisionIDs := make([]uuid.UUID, len(revisions))
	for i := range revisions {
		revisionIDs[i] = revisions[i].RevisionID
	}
	segments, err := loadScheduleEvidenceSegments(ctx, db, revisionIDs)
	if err != nil {
		return api.ScheduleEvidenceResponse{}, err
	}
	closures, err := loadScheduleEvidenceClosures(ctx, db, employmentID, businessDate)
	if err != nil {
		return api.ScheduleEvidenceResponse{}, err
	}
	audits, events, err := loadScheduleEvidenceAudit(ctx, db, revisionIDs)
	if err != nil {
		return api.ScheduleEvidenceResponse{}, err
	}
	if err := writeScheduleEvidenceExportAudit(ctx, db, actorID, employmentID, businessDate, len(revisions)+len(segments)+len(closures)); err != nil {
		return api.ScheduleEvidenceResponse{}, err
	}
	diffChain := make([]api.ScheduleEvidenceDiffLink, 0, len(revisions))
	for _, revision := range revisions {
		diffChain = append(diffChain, api.ScheduleEvidenceDiffLink{RevisionID: revision.RevisionID, PreviousRevisionID: revision.PreviousRevisionID})
	}
	return api.ScheduleEvidenceResponse{
		EmploymentID: employmentID, BusinessDate: businessDate, Revisions: revisions,
		Segments: segments, ClosureDays: closures, DiffChain: diffChain, AuditActions: audits, Events: events,
	}, nil
}

func (r ScheduleRepository) loadScheduleEvidenceRevisions(ctx context.Context, db DBTX, employmentID uuid.UUID, businessDate string) ([]api.ScheduleHistoryEntry, error) {
	rows, err := db.Query(ctx, `SELECT id, employment_id, employer_id, business_date::text, timezone,
		revision_number, state, previous_revision_id, publication_batch_id, source, source_snapshot,
		resolver_version, reason, published_by, published_actor_type, published_actor_name, published_at, created_at
		FROM schedule_day_revisions
		WHERE employment_id = $1 AND business_date = $2::date
		ORDER BY revision_number ASC, id ASC`, employmentID, businessDate)
	if err != nil {
		if isScheduleLifecycleSchemaError(err) {
			return nil, api.ErrScheduleEvidenceUnavailable
		}
		return nil, err
	}
	defer rows.Close()
	result := make([]api.ScheduleHistoryEntry, 0)
	for rows.Next() {
		entry, err := scanScheduleHistoryEntry(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, entry)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	return result, nil
}

func loadScheduleEvidenceSegments(ctx context.Context, db DBTX, revisionIDs []uuid.UUID) ([]api.ScheduleEvidenceSegment, error) {
	rows, err := db.Query(ctx, `SELECT id, employment_id, date::text, schedule_revision_id,
		selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot, sort_order
		FROM work_segments
		WHERE schedule_revision_id = ANY($1)
		ORDER BY date ASC, sort_order ASC, id ASC`, revisionIDs)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.ScheduleEvidenceSegment, 0)
	for rows.Next() {
		var segment api.ScheduleEvidenceSegment
		if err := rows.Scan(&segment.ID, &segment.EmploymentID, &segment.BusinessDate, &segment.ScheduleRevisionID,
			&segment.SelectedShiftVersionID, &segment.ScheduledStartAt, &segment.ScheduledEndAt, &segment.BreaksSnapshot, &segment.SortOrder); err != nil {
			return nil, err
		}
		result = append(result, segment)
	}
	return result, rows.Err()
}

func loadScheduleEvidenceClosures(ctx context.Context, db DBTX, employmentID uuid.UUID, businessDate string) ([]api.ScheduleEvidenceClosureDay, error) {
	rows, err := db.Query(ctx, `SELECT id, closure_id, period_id, employer_id, employment_id, business_date::text,
		schedule_revision_id, schedule_state
		FROM attendance_closure_days
		WHERE employment_id = $1 AND business_date = $2::date
		ORDER BY created_at ASC, id ASC`, employmentID, businessDate)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.ScheduleEvidenceClosureDay, 0)
	for rows.Next() {
		var closure api.ScheduleEvidenceClosureDay
		if err := rows.Scan(&closure.ID, &closure.ClosureID, &closure.PeriodID, &closure.EmployerID, &closure.EmploymentID,
			&closure.BusinessDate, &closure.ScheduleRevisionID, &closure.ScheduleState); err != nil {
			return nil, err
		}
		result = append(result, closure)
	}
	return result, rows.Err()
}

func loadScheduleEvidenceAudit(ctx context.Context, db DBTX, revisionIDs []uuid.UUID) ([]api.ScheduleEvidenceAuditAction, []api.ScheduleEvidenceEvent, error) {
	rows, err := db.Query(ctx, `SELECT id, action, actor_id, actor_type, actor_name, created_at
		FROM audit_logs
		WHERE target_resource_id = ANY($1)
		ORDER BY created_at ASC, id ASC`, revisionIDs)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()
	actions := make([]api.ScheduleEvidenceAuditAction, 0)
	events := make([]api.ScheduleEvidenceEvent, 0)
	for rows.Next() {
		var action api.ScheduleEvidenceAuditAction
		if err := rows.Scan(&action.ID, &action.Action, &action.ActorID, &action.ActorType, &action.ActorName, &action.CreatedAt); err != nil {
			return nil, nil, err
		}
		actions = append(actions, action)
		if action.Action == "notification_delivery" || action.Action == "acknowledgement" {
			events = append(events, api.ScheduleEvidenceEvent{ID: action.ID, Type: action.Action, Action: action.Action, ActorID: action.ActorID, ActorType: action.ActorType, ActorName: action.ActorName, CreatedAt: action.CreatedAt})
		}
	}
	return actions, events, rows.Err()
}

func writeScheduleEvidenceExportAudit(ctx context.Context, db DBTX, actorID, employmentID uuid.UUID, businessDate string, rowCount int) error {
	details, err := json.Marshal(map[string]any{
		"employment_id": employmentID.String(), "business_date": businessDate, "row_count": rowCount,
	})
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, actor_type, actor_name, created_at)
		VALUES ($1, $2, 'schedule.evidence.exported', $3, 'human', NULL, CURRENT_TIMESTAMP)`, actorID, employmentID, details)
	return err
}

func scanScheduleHistoryEntry(row interface{ Scan(...any) error }) (api.ScheduleHistoryEntry, error) {
	var entry api.ScheduleHistoryEntry
	if err := row.Scan(&entry.RevisionID, &entry.EmploymentID, &entry.EmployerID, &entry.BusinessDate, &entry.Timezone,
		&entry.RevisionNumber, &entry.State, &entry.PreviousRevisionID, &entry.PublicationBatchID, &entry.Source,
		&entry.SourceSnapshot, &entry.ResolverVersion, &entry.Reason, &entry.PublishedBy, &entry.PublishedActorType,
		&entry.PublishedActorName, &entry.PublishedAt, &entry.CreatedAt); err != nil {
		return api.ScheduleHistoryEntry{}, err
	}
	return entry, nil
}

func encodeHistoryCursor(cursor api.ScheduleHistoryCursor) (string, error) {
	payload, err := json.Marshal(cursor)
	if err != nil {
		return "", err
	}
	return base64.RawURLEncoding.EncodeToString(payload), nil
}

func isScheduleLifecycleSchemaError(err error) bool {
	if err == nil {
		return false
	}
	message := strings.ToLower(err.Error())
	return strings.Contains(message, "schedule_day_revisions") || strings.Contains(message, "attendance_closure_days") || strings.Contains(message, "schedule lifecycle")
}

var _ api.ScheduleEvidenceRepository = ScheduleRepository{}
