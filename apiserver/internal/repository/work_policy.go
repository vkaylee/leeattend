package repository

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type WorkPolicyRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r WorkPolicyRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "work policy repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("work policy repository: database does not support transactions")
	}
	return beginner, nil
}

const workPolicyVersionSelectColumns = `
	v.id, v.group_id, g.code, g.name, v.version_no, v.status, v.row_version,
	v.effective_from::text, v.effective_to::text,
	v.minutes_per_credit, v.work_unit_minutes, v.rounding_policy, v.early_arrival_strategy,
	v.late_arrival_grace_minutes, v.early_ot_enabled, v.early_ot_qualification_grace_minutes,
	v.early_ot_max_minutes, v.break_auto_deduct, v.break_is_overtime, v.ot_aggregation_mode,
	v.working_pattern_id, v.pay_method, v.monthly_denominator, v.monthly_standard_days::text,
	v.rate_component_definition_id, v.paid_leave_credit_mode, v.overtime_policy_version_id,
	v.created_by, v.created_at, v.updated_at, v.published_by, v.published_at
`

func scanWorkPolicyVersion(row interface{ Scan(...any) error }) (api.WorkPolicyVersion, error) {
	var (
		item                         api.WorkPolicyVersion
		status, payMethod, paidLeave string
		denominator, days            *string
		createdAt, updatedAt         time.Time
		publishedAt                  *time.Time
	)
	if err := row.Scan(
		&item.ID, &item.GroupID, &item.GroupCode, &item.GroupName, &item.VersionNo, &status, &item.RowVersion,
		&item.EffectiveFrom, &item.EffectiveTo,
		&item.MinutesPerCredit, &item.WorkUnitMinutes, &item.RoundingPolicy, &item.EarlyArrivalStrategy,
		&item.LateArrivalGraceMinutes, &item.EarlyOTEnabled, &item.EarlyOTQualificationGraceMinutes,
		&item.EarlyOTMaxMinutes, &item.BreakAutoDeduct, &item.BreakIsOvertime, &item.OTAggregationMode,
		&item.WorkingPatternID, &payMethod, &denominator, &days,
		&item.RateComponentDefinitionID, &paidLeave, &item.OvertimePolicyVersionID,
		&item.CreatedBy, &createdAt, &updatedAt, &item.PublishedBy, &publishedAt,
	); err != nil {
		return api.WorkPolicyVersion{}, err
	}
	return hydrateWorkPolicyVersion(item, status, payMethod, paidLeave, denominator, days, createdAt, updatedAt, publishedAt)
}

func hydrateWorkPolicyVersion(item api.WorkPolicyVersion, status, payMethod, paidLeave string, denominator, days *string, createdAt, updatedAt time.Time, publishedAt *time.Time) (api.WorkPolicyVersion, error) {
	item.Status = workpolicy.VersionStatus(status)
	item.PayMethod = workpolicy.PayMethod(payMethod)
	item.PaidLeaveCreditMode = workpolicy.PaidLeaveCreditMode(paidLeave)
	if denominator != nil && *denominator != "" {
		v := workpolicy.MonthlyDenominator(*denominator)
		item.MonthlyDenominator = &v
	}
	var err error
	if item.MonthlyStandardDays, err = parseOptionalDecimal(days); err != nil {
		return api.WorkPolicyVersion{}, err
	}
	item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	item.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	if publishedAt != nil {
		v := publishedAt.UTC().Format(time.RFC3339)
		item.PublishedAt = &v
	}
	return item, nil
}

func (r WorkPolicyRepository) ListWorkPolicyGroups(ctx context.Context, page, limit int) (api.WorkPolicyGroupListResponse, error) {
	db, err := requireDB(r.DB, "work policy repository")
	if err != nil {
		return api.WorkPolicyGroupListResponse{}, err
	}
	out := api.WorkPolicyGroupListResponse{Data: make([]api.WorkPolicyGroupSummary, 0), Page: page, Limit: limit}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM work_policy_groups`).Scan(&out.Total); err != nil {
		return out, err
	}
	groupIDs, indexByID, err := queryWorkPolicyGroupPage(ctx, db, limit, (page-1)*limit, &out)
	if err != nil || len(groupIDs) == 0 {
		return out, err
	}
	err = attachLatestGroupVersions(ctx, db, groupIDs, indexByID, out.Data)
	return out, err
}

func queryWorkPolicyGroupPage(ctx context.Context, db DBTX, limit, offset int, out *api.WorkPolicyGroupListResponse) ([]uuid.UUID, map[uuid.UUID]int, error) {
	rows, err := db.Query(ctx, `
		SELECT g.id, g.code, g.name, g.description, g.archived_at, g.created_at, g.updated_at,
		       (SELECT count(*) FROM work_policy_memberships m
		        WHERE m.group_id = g.id AND (m.effective_to IS NULL OR m.effective_to > CURRENT_DATE))
		FROM work_policy_groups g
		ORDER BY g.created_at DESC, g.id DESC
		LIMIT $1 OFFSET $2`, limit, offset)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()
	groupIDs := make([]uuid.UUID, 0)
	indexByID := make(map[uuid.UUID]int)
	for rows.Next() {
		var item api.WorkPolicyGroupSummary
		var archivedAt *time.Time
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&item.ID, &item.Code, &item.Name, &item.Description, &archivedAt, &createdAt, &updatedAt, &item.ActiveMembershipCount); err != nil {
			return nil, nil, err
		}
		if archivedAt != nil {
			v := archivedAt.UTC().Format(time.RFC3339)
			item.ArchivedAt = &v
		}
		item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		item.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		indexByID[item.ID] = len(out.Data)
		groupIDs = append(groupIDs, item.ID)
		out.Data = append(out.Data, item)
	}
	return groupIDs, indexByID, rows.Err()
}

func attachLatestGroupVersions(ctx context.Context, db DBTX, groupIDs []uuid.UUID, indexByID map[uuid.UUID]int, data []api.WorkPolicyGroupSummary) error {
	vRows, err := db.Query(ctx, `
		SELECT `+workPolicyVersionSelectColumns+`
		FROM (
			(SELECT DISTINCT ON (group_id) *
			 FROM work_policy_versions
			 WHERE group_id = ANY($1::uuid[]) AND status = 'published'
			 ORDER BY group_id, effective_from DESC, version_no DESC, id DESC)
			UNION ALL
			(SELECT DISTINCT ON (group_id) *
			 FROM work_policy_versions
			 WHERE group_id = ANY($1::uuid[]) AND status = 'draft'
			 ORDER BY group_id, version_no DESC, id DESC)
		) v
		JOIN work_policy_groups g ON g.id = v.group_id`, groupIDs)
	if err != nil {
		return err
	}
	defer vRows.Close()
	for vRows.Next() {
		v, err := scanWorkPolicyVersion(vRows)
		if err != nil {
			return err
		}
		idx, ok := indexByID[v.GroupID]
		if !ok {
			continue
		}
		copyV := v
		if v.Status == workpolicy.StatusPublished {
			data[idx].CurrentPublished = &copyV
		} else if v.Status == workpolicy.StatusDraft {
			data[idx].LatestDraft = &copyV
		}
	}
	return vRows.Err()
}

func (r WorkPolicyRepository) ListWorkPolicyGroupVersions(ctx context.Context, groupID workpolicy.GroupID, statusFilter string, page, limit int) (api.WorkPolicyVersionListResponse, error) {
	db, err := requireDB(r.DB, "work policy repository")
	if err != nil {
		return api.WorkPolicyVersionListResponse{}, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM work_policy_groups WHERE id = $1)`, uuid.UUID(groupID)).Scan(&exists); err != nil {
		return api.WorkPolicyVersionListResponse{}, err
	}
	if !exists {
		return api.WorkPolicyVersionListResponse{}, api.ErrWorkPolicyGroupNotFound
	}
	out := api.WorkPolicyVersionListResponse{Data: make([]api.WorkPolicyVersion, 0), Page: page, Limit: limit}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM work_policy_versions WHERE group_id = $1 AND ($2 = '' OR status = $2)`, uuid.UUID(groupID), statusFilter).Scan(&out.Total); err != nil {
		return out, err
	}
	rows, err := db.Query(ctx, `
		SELECT `+workPolicyVersionSelectColumns+`
		FROM work_policy_versions v
		JOIN work_policy_groups g ON g.id = v.group_id
		WHERE v.group_id = $1 AND ($2 = '' OR v.status = $2)
		ORDER BY v.version_no DESC, v.id DESC
		LIMIT $3 OFFSET $4`, uuid.UUID(groupID), statusFilter, limit, (page-1)*limit)
	if err != nil {
		return out, err
	}
	defer rows.Close()
	for rows.Next() {
		item, err := scanWorkPolicyVersion(rows)
		if err != nil {
			return out, err
		}
		out.Data = append(out.Data, item)
	}
	return out, rows.Err()
}

func (r WorkPolicyRepository) GetWorkPolicyVersion(ctx context.Context, groupID workpolicy.GroupID, versionID workpolicy.VersionID) (api.WorkPolicyVersion, error) {
	db, err := requireDB(r.DB, "work policy repository")
	if err != nil {
		return api.WorkPolicyVersion{}, err
	}
	return fetchWorkPolicyVersion(ctx, db, uuid.UUID(groupID), uuid.UUID(versionID))
}

func fetchWorkPolicyVersion(ctx context.Context, db DBTX, groupID, versionID uuid.UUID) (api.WorkPolicyVersion, error) {
	row := db.QueryRow(ctx, `
		SELECT `+workPolicyVersionSelectColumns+`
		FROM work_policy_versions v
		JOIN work_policy_groups g ON g.id = v.group_id
		WHERE v.group_id = $1 AND v.id = $2`, groupID, versionID)
	item, err := scanWorkPolicyVersion(row)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.WorkPolicyVersion{}, api.ErrWorkPolicyGroupNotFound
	}
	return item, err
}

func (r WorkPolicyRepository) ValidateWorkPolicyVersion(ctx context.Context, groupID workpolicy.GroupID, versionID workpolicy.VersionID) (api.WorkPolicyValidationResponse, error) {
	db, err := requireDB(r.DB, "work policy repository")
	if err != nil {
		return api.WorkPolicyValidationResponse{}, err
	}
	version, err := fetchWorkPolicyVersion(ctx, db, uuid.UUID(groupID), uuid.UUID(versionID))
	if err != nil {
		return api.WorkPolicyValidationResponse{}, err
	}
	return r.validateVersionAgainstDB(ctx, db, version, false)
}

func (r WorkPolicyRepository) validateVersionAgainstDB(ctx context.Context, db DBTX, version api.WorkPolicyVersion, autoClosePrevious bool) (api.WorkPolicyValidationResponse, error) {
	rules, err := versionToDomainRules(version)
	if err != nil {
		return api.WorkPolicyValidationResponse{}, err
	}
	blockers, warnings := rules.Validate()
	refBlockers, err := validateVersionReferences(ctx, db, version)
	if err != nil {
		return api.WorkPolicyValidationResponse{}, err
	}
	blockers = append(blockers, refBlockers...)
	conflictBlockers, err := validateVersionScheduleAndLocks(ctx, db, version, autoClosePrevious)
	if err != nil {
		return api.WorkPolicyValidationResponse{}, err
	}
	blockers = append(blockers, conflictBlockers...)
	if blockers == nil {
		blockers = make([]workpolicy.ValidationIssue, 0)
	}
	if warnings == nil {
		warnings = make([]workpolicy.ValidationIssue, 0)
	}
	return api.WorkPolicyValidationResponse{
		VersionID: version.ID, Valid: len(blockers) == 0,
		Blockers: blockers, Warnings: warnings, CheckedAt: time.Now().UTC().Format(time.RFC3339),
	}, nil
}

func versionToDomainRules(v api.WorkPolicyVersion) (workpolicy.VersionRules, error) {
	from, to, err := parseWorkPolicyRange(v.EffectiveFrom, v.EffectiveTo)
	if err != nil {
		return workpolicy.VersionRules{}, err
	}
	return workpolicy.VersionRules{
		EffectiveFrom: *from, EffectiveTo: to,
		MinutesPerCredit: v.MinutesPerCredit, WorkUnitMinutes: v.WorkUnitMinutes,
		RoundingPolicy: v.RoundingPolicy, EarlyArrivalStrategy: v.EarlyArrivalStrategy,
		LateArrivalGraceMinutes: v.LateArrivalGraceMinutes, EarlyOTEnabled: v.EarlyOTEnabled,
		EarlyOTQualificationGraceMinutes: v.EarlyOTQualificationGraceMinutes, EarlyOTMaxMinutes: v.EarlyOTMaxMinutes,
		BreakAutoDeduct: v.BreakAutoDeduct, BreakIsOvertime: v.BreakIsOvertime, OTAggregationMode: v.OTAggregationMode,
		WorkingPatternID: v.WorkingPatternID, PayMethod: v.PayMethod,
		MonthlyDenominator: v.MonthlyDenominator, MonthlyStandardDays: v.MonthlyStandardDays,
		RateComponentDefinitionID: v.RateComponentDefinitionID, PaidLeaveCreditMode: v.PaidLeaveCreditMode,
		OvertimePolicyVersionID: v.OvertimePolicyVersionID,
	}, nil
}

func validateVersionReferences(ctx context.Context, db DBTX, v api.WorkPolicyVersion) ([]workpolicy.ValidationIssue, error) {
	var blockers []workpolicy.ValidationIssue
	otBlockers, err := checkOTVersionPublishable(ctx, db, v.OvertimePolicyVersionID)
	if err != nil {
		return nil, err
	}
	blockers = append(blockers, otBlockers...)
	if v.WorkingPatternID != nil && *v.WorkingPatternID != uuid.Nil {
		active, err := rowExists(ctx, db, `SELECT EXISTS (SELECT 1 FROM working_patterns WHERE id = $1 AND is_active = TRUE)`, *v.WorkingPatternID)
		if err != nil {
			return nil, err
		}
		if !active {
			blockers = append(blockers, workpolicy.ValidationIssue{Code: "working_pattern_inactive_or_missing", Field: "working_pattern_id", Severity: "blocker", Message: "referenced working pattern is missing or inactive"})
		}
	}
	if v.RateComponentDefinitionID != nil && *v.RateComponentDefinitionID != uuid.Nil {
		ok, err := rowExists(ctx, db, `SELECT EXISTS (SELECT 1 FROM salary_component_definitions WHERE id = $1)`, *v.RateComponentDefinitionID)
		if err != nil {
			return nil, err
		}
		if !ok {
			blockers = append(blockers, workpolicy.ValidationIssue{Code: "rate_component_not_found", Field: "rate_component_definition_id", Severity: "blocker", Message: "referenced salary rate component does not exist"})
		}
	}
	return blockers, nil
}

func checkOTVersionPublishable(ctx context.Context, db DBTX, otVersionID uuid.UUID) ([]workpolicy.ValidationIssue, error) {
	var otStatus string
	var otRulesRaw []byte
	err := db.QueryRow(ctx, `SELECT status, rules FROM overtime_policy_versions WHERE id = $1`, otVersionID).Scan(&otStatus, &otRulesRaw)
	if errors.Is(err, pgx.ErrNoRows) {
		return []workpolicy.ValidationIssue{{Code: "overtime_policy_version_not_found", Field: "overtime_policy_version_id", Severity: "blocker", Message: "referenced overtime policy version does not exist"}}, nil
	}
	if err != nil {
		return nil, err
	}
	var blockers []workpolicy.ValidationIssue
	if otStatus != string(workpolicy.StatusPublished) {
		blockers = append(blockers, workpolicy.ValidationIssue{Code: "overtime_policy_version_not_published", Field: "overtime_policy_version_id", Severity: "blocker", Message: "referenced overtime policy version must be published before publishing work policy"})
	}
	var otRules []workpolicy.OvertimeRule
	if json.Unmarshal(otRulesRaw, &otRules) != nil || len(otRules) == 0 {
		blockers = append(blockers, workpolicy.ValidationIssue{Code: "overtime_policy_rules_empty", Field: "overtime_policy_version_id", Severity: "blocker", Message: "referenced overtime policy version must contain at least one valid rule"})
	}
	return blockers, nil
}

func validateVersionScheduleAndLocks(ctx context.Context, db DBTX, v api.WorkPolicyVersion, autoClosePrevious bool) ([]workpolicy.ValidationIssue, error) {
	var blockers []workpolicy.ValidationIssue
	if v.Status == workpolicy.StatusDraft {
		overlaps, err := hasPublishedVersionOverlap(ctx, db, v, autoClosePrevious)
		if err != nil {
			return nil, err
		}
		if overlaps {
			blockers = append(blockers, workpolicy.ValidationIssue{Code: "published_version_overlap", Field: "effective_from", Severity: "blocker", Message: "effective range overlaps an existing published version in this group"})
		}
	}
	touchesLocked, err := versionTouchesLockedPayrollPeriod(ctx, db, v.GroupID, v.EffectiveFrom, v.EffectiveTo)
	if err != nil {
		return nil, err
	}
	if touchesLocked {
		blockers = append(blockers, workpolicy.ValidationIssue{Code: "locked_payroll_period", Field: "effective_from", Severity: "blocker", Message: "effective range intersects a locked payroll period for assigned employees"})
	}
	return blockers, nil
}

func hasPublishedVersionOverlap(ctx context.Context, db DBTX, v api.WorkPolicyVersion, autoClosePrevious bool) (bool, error) {
	extra := ""
	if autoClosePrevious {
		extra = " AND effective_from >= $3::date"
	}
	var count int
	err := db.QueryRow(ctx, `
		SELECT count(*) FROM work_policy_versions
		WHERE group_id = $1 AND status = 'published' AND id <> $2`+extra+`
		  AND daterange(effective_from, COALESCE(effective_to, 'infinity'::date), '[)')
		      && daterange($3::date, COALESCE($4::date, 'infinity'::date), '[)')`,
		v.GroupID, v.ID, v.EffectiveFrom, v.EffectiveTo).Scan(&count)
	return count > 0, err
}

func versionTouchesLockedPayrollPeriod(ctx context.Context, db DBTX, groupID uuid.UUID, effFrom string, effTo *string) (bool, error) {
	return rowExists(ctx, db, `
		SELECT EXISTS (
			SELECT 1
			FROM work_policy_memberships m
			JOIN employment_relationships er ON er.id = m.employment_relationship_id AND er.deleted_at IS NULL
			JOIN payroll_periods pp ON pp.employer_id = er.employer_id AND pp.status = 'locked'
			WHERE m.group_id = $1
			  AND daterange(m.effective_from, COALESCE(m.effective_to, 'infinity'::date), '[)')
			      && daterange($2::date, COALESCE($3::date, 'infinity'::date), '[)')
			  AND daterange(make_date(pp.period_year, pp.period_month, 1), (make_date(pp.period_year, pp.period_month, 1) + INTERVAL '1 month')::date, '[)')
			      && daterange($2::date, COALESCE($3::date, 'infinity'::date), '[)')
		)`, groupID, effFrom, effTo)
}

func rowExists(ctx context.Context, db DBTX, query string, args ...any) (bool, error) {
	var ok bool
	err := db.QueryRow(ctx, query, args...).Scan(&ok)
	return ok, err
}

func (r WorkPolicyRepository) verifyReferencesExist(ctx context.Context, db DBTX, draft workpolicy.VersionDraftFields) error {
	checks := []struct {
		id    *uuid.UUID
		query string
	}{
		{id: &draft.OvertimePolicyVersionID, query: `SELECT EXISTS (SELECT 1 FROM overtime_policy_versions WHERE id = $1)`},
		{id: draft.WorkingPatternID, query: `SELECT EXISTS (SELECT 1 FROM working_patterns WHERE id = $1 AND is_active = TRUE)`},
		{id: draft.RateComponentDefinitionID, query: `SELECT EXISTS (SELECT 1 FROM salary_component_definitions WHERE id = $1)`},
	}
	for _, c := range checks {
		if c.id == nil || *c.id == uuid.Nil {
			continue
		}
		ok, err := rowExists(ctx, db, c.query, *c.id)
		if err != nil {
			return err
		}
		if !ok {
			return api.ErrWorkPolicyReferenceNotFound
		}
	}
	return nil
}

func insertDraftWorkPolicyVersion(ctx context.Context, tx pgx.Tx, groupID uuid.UUID, versionNo int, draft workpolicy.VersionDraftFields, from, to *time.Time, actorID uuid.UUID) (api.WorkPolicyVersion, error) {
	versionID := uuid.New()
	if _, err := tx.Exec(ctx, `
		INSERT INTO work_policy_versions (
			id, group_id, version_no, status, row_version, effective_from, effective_to, created_by,
			minutes_per_credit, work_unit_minutes, rounding_policy, early_arrival_strategy,
			late_arrival_grace_minutes, early_ot_enabled, early_ot_qualification_grace_minutes,
			early_ot_max_minutes, break_auto_deduct, break_is_overtime, ot_aggregation_mode,
			working_pattern_id, pay_method, monthly_denominator, monthly_standard_days,
			rate_component_definition_id, paid_leave_credit_mode, overtime_policy_version_id
		) VALUES ($1,$2,$3,'draft',1,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24)`,
		versionID, groupID, versionNo, from, to, actorID,
		draft.MinutesPerCredit, draft.WorkUnitMinutes, draft.RoundingPolicy, draft.EarlyArrivalStrategy,
		draft.LateArrivalGraceMinutes, draft.EarlyOTEnabled, draft.EarlyOTQualificationGraceMinutes,
		draft.EarlyOTMaxMinutes, draft.BreakAutoDeduct, draft.BreakIsOvertime, draft.OTAggregationMode,
		draft.WorkingPatternID, string(draft.PayMethod), monthlyDenominatorPtr(draft.MonthlyDenominator), draft.MonthlyStandardDays,
		draft.RateComponentDefinitionID, string(draft.PaidLeaveCreditMode), draft.OvertimePolicyVersionID,
	); err != nil {
		return api.WorkPolicyVersion{}, mapWorkPolicyError(err)
	}
	return fetchWorkPolicyVersion(ctx, tx, groupID, versionID)
}

func (r WorkPolicyRepository) CreateWorkPolicyGroup(ctx context.Context, req workpolicy.CreateGroupInput, actorID ids.UserID) (api.WorkPolicyVersion, error) {
	db, err := requireDB(r.DB, "work policy repository")
	if err != nil {
		return api.WorkPolicyVersion{}, err
	}
	if err := r.verifyReferencesExist(ctx, db, req.Draft); err != nil {
		return api.WorkPolicyVersion{}, err
	}
	from, to, err := parseWorkPolicyRange(req.Draft.EffectiveFrom, req.Draft.EffectiveTo)
	if err != nil {
		return api.WorkPolicyVersion{}, api.ErrWorkPolicyGroupValidation
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.WorkPolicyVersion{}, err
	}
	var created api.WorkPolicyVersion
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		groupID := uuid.New()
		if _, err := tx.Exec(ctx, `INSERT INTO work_policy_groups (id, code, name, description, created_by) VALUES ($1, $2, $3, $4, $5)`,
			groupID, strings.TrimSpace(req.Code), strings.TrimSpace(req.Name), req.Description, uuid.UUID(actorID)); err != nil {
			return mapWorkPolicyError(err)
		}
		created, err = insertDraftWorkPolicyVersion(ctx, tx, groupID, 1, req.Draft, from, to, uuid.UUID(actorID))
		return err
	})
	return created, err
}

func draftFromVersion(v api.WorkPolicyVersion) workpolicy.VersionDraftFields {
	return workpolicy.VersionDraftFields{
		EffectiveFrom: v.EffectiveFrom, EffectiveTo: v.EffectiveTo,
		MinutesPerCredit: v.MinutesPerCredit, WorkUnitMinutes: v.WorkUnitMinutes,
		RoundingPolicy: v.RoundingPolicy, EarlyArrivalStrategy: v.EarlyArrivalStrategy,
		LateArrivalGraceMinutes: v.LateArrivalGraceMinutes, EarlyOTEnabled: v.EarlyOTEnabled,
		EarlyOTQualificationGraceMinutes: v.EarlyOTQualificationGraceMinutes, EarlyOTMaxMinutes: v.EarlyOTMaxMinutes,
		BreakAutoDeduct: v.BreakAutoDeduct, BreakIsOvertime: v.BreakIsOvertime, OTAggregationMode: v.OTAggregationMode,
		WorkingPatternID: v.WorkingPatternID, PayMethod: v.PayMethod,
		MonthlyDenominator: v.MonthlyDenominator, MonthlyStandardDays: v.MonthlyStandardDays,
		RateComponentDefinitionID: v.RateComponentDefinitionID, PaidLeaveCreditMode: v.PaidLeaveCreditMode,
		OvertimePolicyVersionID: v.OvertimePolicyVersionID,
	}
}

func lockActiveGroupForUpdate(ctx context.Context, tx pgx.Tx, groupID uuid.UUID) error {
	var archivedAt *time.Time
	if err := tx.QueryRow(ctx, `SELECT archived_at FROM work_policy_groups WHERE id = $1 FOR UPDATE`, groupID).Scan(&archivedAt); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrWorkPolicyGroupNotFound
		}
		return err
	}
	if archivedAt != nil {
		return api.ErrWorkPolicyVersionConflict
	}
	return nil
}

func (r WorkPolicyRepository) CreateWorkPolicyVersion(ctx context.Context, req workpolicy.CreateVersionInput, actorID ids.UserID) (api.WorkPolicyVersion, error) {
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.WorkPolicyVersion{}, err
	}
	var created api.WorkPolicyVersion
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := lockActiveGroupForUpdate(ctx, tx, uuid.UUID(req.GroupID)); err != nil {
			return err
		}
		draft, err := resolveCreateVersionDraft(ctx, tx, req)
		if err != nil {
			return err
		}
		if err := r.verifyReferencesExist(ctx, tx, draft); err != nil {
			return err
		}
		from, to, err := parseWorkPolicyRange(draft.EffectiveFrom, draft.EffectiveTo)
		if err != nil {
			return api.ErrWorkPolicyGroupValidation
		}
		var nextVersion int
		if err := tx.QueryRow(ctx, `SELECT COALESCE(MAX(version_no), 0) + 1 FROM work_policy_versions WHERE group_id = $1`, uuid.UUID(req.GroupID)).Scan(&nextVersion); err != nil {
			return err
		}
		created, err = insertDraftWorkPolicyVersion(ctx, tx, uuid.UUID(req.GroupID), nextVersion, draft, from, to, uuid.UUID(actorID))
		return err
	})
	return created, err
}

func resolveCreateVersionDraft(ctx context.Context, tx pgx.Tx, req workpolicy.CreateVersionInput) (workpolicy.VersionDraftFields, error) {
	if req.Draft != nil {
		return *req.Draft, nil
	}
	source, err := fetchWorkPolicyVersion(ctx, tx, uuid.UUID(req.GroupID), *req.CloneFromVersionID)
	if err != nil {
		return workpolicy.VersionDraftFields{}, err
	}
	return draftFromVersion(source), nil
}

func (r WorkPolicyRepository) PatchWorkPolicyVersion(ctx context.Context, req workpolicy.PatchVersionInput) (api.WorkPolicyVersion, error) {
	from, to, err := parseWorkPolicyRange(req.Draft.EffectiveFrom, req.Draft.EffectiveTo)
	if err != nil {
		return api.WorkPolicyVersion{}, api.ErrWorkPolicyGroupValidation
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.WorkPolicyVersion{}, err
	}
	var updated api.WorkPolicyVersion
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := lockDraftVersionForEdit(ctx, tx, uuid.UUID(req.GroupID), uuid.UUID(req.VersionID), req.ExpectedRowVersion); err != nil {
			return err
		}
		if err := r.verifyReferencesExist(ctx, tx, req.Draft); err != nil {
			return err
		}
		updated, err = updateDraftWorkPolicyVersion(ctx, tx, req, from, to)
		return err
	})
	return updated, err
}

func lockDraftVersionForEdit(ctx context.Context, tx pgx.Tx, groupID, versionID uuid.UUID, expectedRowVersion int64) error {
	var status string
	var rowVersion int64
	if err := tx.QueryRow(ctx, `SELECT status, row_version FROM work_policy_versions WHERE group_id = $1 AND id = $2 FOR UPDATE`, groupID, versionID).Scan(&status, &rowVersion); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrWorkPolicyGroupNotFound
		}
		return err
	}
	if status != string(workpolicy.StatusDraft) || rowVersion != expectedRowVersion {
		return api.ErrWorkPolicyVersionConflict
	}
	return nil
}

func updateDraftWorkPolicyVersion(ctx context.Context, tx pgx.Tx, req workpolicy.PatchVersionInput, from, to *time.Time) (api.WorkPolicyVersion, error) {
	tag, err := tx.Exec(ctx, `
		UPDATE work_policy_versions
		SET effective_from=$3, effective_to=$4, minutes_per_credit=$5, work_unit_minutes=$6, rounding_policy=$7,
		    early_arrival_strategy=$8, late_arrival_grace_minutes=$9, early_ot_enabled=$10,
		    early_ot_qualification_grace_minutes=$11, early_ot_max_minutes=$12, break_auto_deduct=$13,
		    break_is_overtime=$14, ot_aggregation_mode=$15, working_pattern_id=$16, pay_method=$17,
		    monthly_denominator=$18, monthly_standard_days=$19, rate_component_definition_id=$20,
		    paid_leave_credit_mode=$21, overtime_policy_version_id=$22, row_version=row_version+1, updated_at=CURRENT_TIMESTAMP
		WHERE group_id=$1 AND id=$2 AND status='draft' AND row_version=$23`,
		uuid.UUID(req.GroupID), uuid.UUID(req.VersionID), from, to,
		req.Draft.MinutesPerCredit, req.Draft.WorkUnitMinutes, req.Draft.RoundingPolicy, req.Draft.EarlyArrivalStrategy,
		req.Draft.LateArrivalGraceMinutes, req.Draft.EarlyOTEnabled, req.Draft.EarlyOTQualificationGraceMinutes,
		req.Draft.EarlyOTMaxMinutes, req.Draft.BreakAutoDeduct, req.Draft.BreakIsOvertime, req.Draft.OTAggregationMode,
		req.Draft.WorkingPatternID, string(req.Draft.PayMethod), monthlyDenominatorPtr(req.Draft.MonthlyDenominator), req.Draft.MonthlyStandardDays,
		req.Draft.RateComponentDefinitionID, string(req.Draft.PaidLeaveCreditMode), req.Draft.OvertimePolicyVersionID,
		req.ExpectedRowVersion)
	if err != nil {
		return api.WorkPolicyVersion{}, mapWorkPolicyError(err)
	}
	if tag.RowsAffected() == 0 {
		return api.WorkPolicyVersion{}, api.ErrWorkPolicyVersionConflict
	}
	return fetchWorkPolicyVersion(ctx, tx, uuid.UUID(req.GroupID), uuid.UUID(req.VersionID))
}

func (r WorkPolicyRepository) PublishWorkPolicyVersion(ctx context.Context, req workpolicy.PublishVersionInput) (api.WorkPolicyVersion, error) {
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.WorkPolicyVersion{}, err
	}
	var published api.WorkPolicyVersion
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := lockActiveGroupForUpdate(ctx, tx, uuid.UUID(req.GroupID)); err != nil {
			return err
		}
		replayed, done, err := checkPublishIdempotency(ctx, tx, req)
		if err != nil || done {
			published = replayed
			return err
		}
		published, err = r.executeAtomicPublish(ctx, tx, req)
		return err
	})
	return published, err
}

func checkPublishIdempotency(ctx context.Context, tx pgx.Tx, req workpolicy.PublishVersionInput) (api.WorkPolicyVersion, bool, error) {
	if req.IdempotencyKey == "" {
		return api.WorkPolicyVersion{}, false, nil
	}
	var existingID uuid.UUID
	err := tx.QueryRow(ctx, `SELECT id FROM work_policy_versions WHERE group_id = $1 AND publish_idempotency_key = $2`,
		uuid.UUID(req.GroupID), req.IdempotencyKey).Scan(&existingID)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.WorkPolicyVersion{}, false, nil
	}
	if err != nil {
		return api.WorkPolicyVersion{}, false, err
	}
	if existingID != uuid.UUID(req.VersionID) {
		return api.WorkPolicyVersion{}, false, api.ErrWorkPolicyVersionConflict
	}
	v, err := fetchWorkPolicyVersion(ctx, tx, uuid.UUID(req.GroupID), existingID)
	return v, true, err
}

func (r WorkPolicyRepository) executeAtomicPublish(ctx context.Context, tx pgx.Tx, req workpolicy.PublishVersionInput) (api.WorkPolicyVersion, error) {
	current, err := scanWorkPolicyVersion(tx.QueryRow(ctx, `
		SELECT `+workPolicyVersionSelectColumns+`
		FROM work_policy_versions v JOIN work_policy_groups g ON g.id = v.group_id
		WHERE v.group_id = $1 AND v.id = $2 FOR UPDATE OF v`, uuid.UUID(req.GroupID), uuid.UUID(req.VersionID)))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.WorkPolicyVersion{}, api.ErrWorkPolicyGroupNotFound
	}
	if err != nil {
		return api.WorkPolicyVersion{}, err
	}
	if current.Status != workpolicy.StatusDraft || current.RowVersion != req.ExpectedRowVersion {
		return api.WorkPolicyVersion{}, api.ErrWorkPolicyVersionConflict
	}
	validation, err := r.validateVersionAgainstDB(ctx, tx, current, req.AutoClosePrevious)
	if err != nil {
		return api.WorkPolicyVersion{}, err
	}
	if err := mapValidationBlockersError(validation); err != nil {
		return api.WorkPolicyVersion{}, err
	}
	if req.AutoClosePrevious {
		if _, err := tx.Exec(ctx, `
			UPDATE work_policy_versions SET effective_to = $3::date, row_version = row_version + 1, updated_at = CURRENT_TIMESTAMP
			WHERE group_id = $1 AND status = 'published' AND id <> $2 AND effective_from < $3::date AND (effective_to IS NULL OR effective_to > $3::date)`,
			uuid.UUID(req.GroupID), uuid.UUID(req.VersionID), current.EffectiveFrom); err != nil {
			return api.WorkPolicyVersion{}, mapWorkPolicyError(err)
		}
	}
	return markVersionPublished(ctx, tx, req)
}

func mapValidationBlockersError(v api.WorkPolicyValidationResponse) error {
	if v.Valid {
		return nil
	}
	for _, b := range v.Blockers {
		if b.Code == "locked_payroll_period" {
			return api.ErrWorkPolicyLockedPeriod
		}
		if b.Code == "published_version_overlap" {
			return api.ErrWorkPolicyVersionConflict
		}
	}
	return api.ErrWorkPolicyGroupValidation
}

func markVersionPublished(ctx context.Context, tx pgx.Tx, req workpolicy.PublishVersionInput) (api.WorkPolicyVersion, error) {
	var keyArg *string
	if req.IdempotencyKey != "" {
		k := req.IdempotencyKey
		keyArg = &k
	}
	tag, err := tx.Exec(ctx, `
		UPDATE work_policy_versions
		SET status = 'published', published_by = $3, published_at = CURRENT_TIMESTAMP,
		    publish_idempotency_key = $4, row_version = row_version + 1, updated_at = CURRENT_TIMESTAMP
		WHERE group_id = $1 AND id = $2 AND status = 'draft' AND row_version = $5`,
		uuid.UUID(req.GroupID), uuid.UUID(req.VersionID), uuid.UUID(req.ActorID), keyArg, req.ExpectedRowVersion)
	if err != nil {
		return api.WorkPolicyVersion{}, mapWorkPolicyError(err)
	}
	if tag.RowsAffected() == 0 {
		return api.WorkPolicyVersion{}, api.ErrWorkPolicyVersionConflict
	}
	return fetchWorkPolicyVersion(ctx, tx, uuid.UUID(req.GroupID), uuid.UUID(req.VersionID))
}

func (r WorkPolicyRepository) ResolveWorkPolicyByRelationship(ctx context.Context, relationshipID uuid.UUID, asOf time.Time) (api.ResolvedWorkPolicyResponse, error) {
	db, err := requireDB(r.DB, "work policy repository")
	if err != nil {
		return api.ResolvedWorkPolicyResponse{}, err
	}
	resolved, err := (WorkPolicyResolver{}).ResolveAt(ctx, db, relationshipID, asOf)
	if err != nil {
		return api.ResolvedWorkPolicyResponse{}, err
	}
	var effTo *string
	if resolved.EffectiveTo != nil {
		v := resolved.EffectiveTo.Format("2006-01-02")
		effTo = &v
	}
	return api.ResolvedWorkPolicyResponse{
		MembershipID: resolved.MembershipID, EmploymentRelationshipID: resolved.EmploymentRelationshipID,
		GroupID: resolved.GroupID, GroupCode: resolved.GroupCode, GroupName: resolved.GroupName,
		VersionID: resolved.VersionID, VersionNo: resolved.VersionNo, Status: resolved.Status,
		EffectiveFrom: resolved.EffectiveFrom.Format("2006-01-02"), EffectiveTo: effTo,
		MinutesPerCredit: resolved.MinutesPerCredit, WorkUnitMinutes: resolved.WorkUnitMinutes,
		RoundingPolicy: resolved.RoundingPolicy, EarlyArrivalStrategy: resolved.EarlyArrivalStrategy,
		LateArrivalGraceMinutes: resolved.LateArrivalGraceMinutes, EarlyOTEnabled: resolved.EarlyOTEnabled,
		EarlyOTQualificationGraceMinutes: resolved.EarlyOTQualificationGraceMinutes, EarlyOTMaxMinutes: resolved.EarlyOTMaxMinutes,
		BreakAutoDeduct: resolved.BreakAutoDeduct, BreakIsOvertime: resolved.BreakIsOvertime, OTAggregationMode: resolved.OTAggregationMode,
		WorkingPatternID: resolved.WorkingPatternID, PayMethod: resolved.PayMethod,
		MonthlyDenominator: resolved.MonthlyDenominator, MonthlyStandardDays: resolved.MonthlyStandardDays,
		RateComponentDefinitionID: resolved.RateComponentDefinitionID, PaidLeaveCreditMode: resolved.PaidLeaveCreditMode,
		OvertimePolicyVersionID: resolved.OvertimePolicyVersionID, OvertimePolicyID: resolved.OvertimePolicyID,
		OvertimePolicyVersionNo: resolved.OvertimePolicyVersionNo, DailyCapMinutes: resolved.DailyCapMinutes,
		MonthlyCapHours: resolved.MonthlyCapHours, YearlyCapHours: resolved.YearlyCapHours, OvertimeRules: resolved.OvertimeRules,
	}, nil
}

func (r WorkPolicyRepository) ListWorkPolicyMemberships(ctx context.Context, groupID workpolicy.GroupID) ([]api.WorkPolicyMembership, error) {
	db, err := requireDB(r.DB, "work policy repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `
		SELECT id, employment_relationship_id, group_id, effective_from::text, effective_to::text, created_at
		FROM work_policy_memberships WHERE group_id = $1 ORDER BY effective_from DESC, id DESC`, uuid.UUID(groupID))
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.WorkPolicyMembership, 0)
	for rows.Next() {
		var item api.WorkPolicyMembership
		var createdAt time.Time
		if err := rows.Scan(&item.ID, &item.EmploymentRelationshipID, &item.GroupID, &item.EffectiveFrom, &item.EffectiveTo, &createdAt); err != nil {
			return nil, err
		}
		item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		result = append(result, item)
	}
	return result, rows.Err()
}

func lockRelationshipForMembership(ctx context.Context, tx pgx.Tx, relID uuid.UUID) (uuid.UUID, time.Time, *time.Time, error) {
	var employerID uuid.UUID
	var relStart time.Time
	var relEnd *time.Time
	err := tx.QueryRow(ctx, `SELECT employer_id, start_date, end_date FROM employment_relationships WHERE id = $1 AND deleted_at IS NULL FOR UPDATE`, relID).Scan(&employerID, &relStart, &relEnd)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, time.Time{}, nil, api.ErrEmploymentRelationshipNotFound
	}
	return employerID, relStart, relEnd, err
}

func insertWorkPolicyMembershipRow(ctx context.Context, tx pgx.Tx, relID, groupID uuid.UUID, from time.Time, to *time.Time, actorID uuid.UUID) (api.WorkPolicyMembership, error) {
	var item api.WorkPolicyMembership
	var createdAt time.Time
	err := tx.QueryRow(ctx, `
		INSERT INTO work_policy_memberships (id, employment_relationship_id, group_id, effective_from, effective_to, created_by)
		VALUES ($1, $2, $3, $4, $5, $6)
		RETURNING id, employment_relationship_id, group_id, effective_from::text, effective_to::text, created_at`,
		uuid.New(), relID, groupID, from, to, actorID,
	).Scan(&item.ID, &item.EmploymentRelationshipID, &item.GroupID, &item.EffectiveFrom, &item.EffectiveTo, &createdAt)
	if err != nil {
		if isConflictError(err) {
			return api.WorkPolicyMembership{}, api.ErrWorkPolicyMembershipConflict
		}
		return api.WorkPolicyMembership{}, mapWorkPolicyError(err)
	}
	item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	return item, nil
}

func (r WorkPolicyRepository) CreateWorkPolicyMembership(ctx context.Context, req workpolicy.CreateMembershipInput) (api.WorkPolicyMembership, error) {
	from, to, err := parseWorkPolicyRange(req.EffectiveFrom, req.EffectiveTo)
	if err != nil {
		return api.WorkPolicyMembership{}, api.ErrWorkPolicyGroupValidation
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.WorkPolicyMembership{}, err
	}
	var item api.WorkPolicyMembership
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := lockActiveGroupForUpdate(ctx, tx, uuid.UUID(req.GroupID)); err != nil {
			return err
		}
		employerID, relStart, relEnd, err := lockRelationshipForMembership(ctx, tx, req.EmploymentRelationshipID)
		if err != nil {
			return err
		}
		if err := validatePolicyMembershipRange(from, to, relStart, relEnd); err != nil {
			return err
		}
		if err := ensureNoLockedPayrollPeriodOverlap(ctx, tx, employerID, *from, to); err != nil {
			return err
		}
		item, err = insertWorkPolicyMembershipRow(ctx, tx, req.EmploymentRelationshipID, uuid.UUID(req.GroupID), *from, to, uuid.UUID(req.ActorID))
		return err
	})
	return item, err
}

func (r WorkPolicyRepository) TransferWorkPolicyMembership(ctx context.Context, req workpolicy.TransferMembershipInput) (api.TransferWorkPolicyMembershipResponse, error) {
	effectiveDate, err := time.Parse("2006-01-02", strings.TrimSpace(req.EffectiveDate))
	if err != nil {
		return api.TransferWorkPolicyMembershipResponse{}, api.ErrWorkPolicyGroupValidation
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.TransferWorkPolicyMembershipResponse{}, err
	}
	var out api.TransferWorkPolicyMembershipResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := lockActiveGroupForUpdate(ctx, tx, uuid.UUID(req.TargetGroupID)); err != nil {
			return err
		}
		employerID, relStart, relEnd, err := lockRelationshipForMembership(ctx, tx, req.EmploymentRelationshipID)
		if err != nil {
			return err
		}
		newEffectiveTo := exclusiveEndFromRelEnd(relEnd)
		if err := validatePolicyMembershipRange(&effectiveDate, newEffectiveTo, relStart, relEnd); err != nil {
			return err
		}
		if err := ensureNoLockedPayrollPeriodOverlap(ctx, tx, employerID, effectiveDate, newEffectiveTo); err != nil {
			return err
		}
		out.ClosedMembership, err = closePrecedingMembership(ctx, tx, req.EmploymentRelationshipID, effectiveDate)
		if err != nil {
			return err
		}
		out.NewMembership, err = insertWorkPolicyMembershipRow(ctx, tx, req.EmploymentRelationshipID, uuid.UUID(req.TargetGroupID), effectiveDate, newEffectiveTo, uuid.UUID(req.ActorID))
		return err
	})
	return out, err
}

func exclusiveEndFromRelEnd(relEnd *time.Time) *time.Time {
	if relEnd == nil {
		return nil
	}
	v := relEnd.AddDate(0, 0, 1)
	return &v
}

func closePrecedingMembership(ctx context.Context, tx pgx.Tx, relID uuid.UUID, effectiveDate time.Time) (*api.WorkPolicyMembership, error) {
	var closed api.WorkPolicyMembership
	var createdAt time.Time
	err := tx.QueryRow(ctx, `
		UPDATE work_policy_memberships SET effective_to = $2::date
		WHERE employment_relationship_id = $1 AND effective_from < $2::date AND (effective_to IS NULL OR effective_to > $2::date)
		RETURNING id, employment_relationship_id, group_id, effective_from::text, effective_to::text, created_at`,
		relID, effectiveDate.Format("2006-01-02"),
	).Scan(&closed.ID, &closed.EmploymentRelationshipID, &closed.GroupID, &closed.EffectiveFrom, &closed.EffectiveTo, &createdAt)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, mapWorkPolicyError(err)
	}
	closed.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	return &closed, nil
}

func ensureNoLockedPayrollPeriodOverlap(ctx context.Context, db DBTX, employerID uuid.UUID, from time.Time, to *time.Time) error {
	var toArg *string
	if to != nil {
		v := to.Format("2006-01-02")
		toArg = &v
	}
	locked, err := rowExists(ctx, db, `
		SELECT EXISTS (
			SELECT 1 FROM payroll_periods pp
			WHERE pp.employer_id = $1 AND pp.status = 'locked'
			  AND daterange(make_date(pp.period_year, pp.period_month, 1), (make_date(pp.period_year, pp.period_month, 1) + INTERVAL '1 month')::date, '[)')
			      && daterange($2::date, COALESCE($3::date, 'infinity'::date), '[)')
		)`, employerID, from.Format("2006-01-02"), toArg)
	if err != nil {
		return err
	}
	if locked {
		return api.ErrWorkPolicyLockedPeriod
	}
	return nil
}

func validatePolicyMembershipRange(from, to *time.Time, relStart time.Time, relEnd *time.Time) error {
	if from == nil || from.Before(relStart) {
		return api.ErrWorkPolicyGroupValidation
	}
	if relEnd != nil {
		relExclusiveEnd := relEnd.AddDate(0, 0, 1)
		if !from.Before(relExclusiveEnd) || to == nil || to.After(relExclusiveEnd) {
			return api.ErrWorkPolicyGroupValidation
		}
	}
	return nil
}

func parseWorkPolicyRange(fromValue string, toValue *string) (*time.Time, *time.Time, error) {
	from, err := time.Parse("2006-01-02", strings.TrimSpace(fromValue))
	if err != nil {
		return nil, nil, err
	}
	if toValue == nil || strings.TrimSpace(*toValue) == "" {
		return &from, nil, nil
	}
	to, err := time.Parse("2006-01-02", strings.TrimSpace(*toValue))
	if err != nil {
		return nil, nil, err
	}
	if !to.After(from) {
		return nil, nil, api.ErrWorkPolicyGroupValidation
	}
	return &from, &to, nil
}

func monthlyDenominatorPtr(v *workpolicy.MonthlyDenominator) *string {
	if v == nil {
		return nil
	}
	s := string(*v)
	return &s
}

func mapWorkPolicyError(err error) error {
	if isConflictError(err) {
		return api.ErrWorkPolicyVersionConflict
	}
	return mapConstraintError(err)
}

func closeRelationshipPolicyMemberships(ctx context.Context, tx pgx.Tx, relationshipIDs []uuid.UUID, endDate string) error {
	if len(relationshipIDs) == 0 {
		return nil
	}
	if _, err := tx.Exec(ctx, `DELETE FROM work_policy_memberships WHERE employment_relationship_id = ANY($1::uuid[]) AND effective_from > $2::date`, relationshipIDs, endDate); err != nil {
		return err
	}
	_, err := tx.Exec(ctx, `UPDATE work_policy_memberships SET effective_to = ($2::date + 1) WHERE employment_relationship_id = ANY($1::uuid[]) AND (effective_to IS NULL OR effective_to > ($2::date + 1))`, relationshipIDs, endDate)
	return err
}
