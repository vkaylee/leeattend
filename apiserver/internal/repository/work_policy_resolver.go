package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

// WorkPolicyResolver is the single canonical resolver for published work policies
// across Attendance, Reports, Payroll, and Simulation.
type WorkPolicyResolver struct{}

const resolvedPolicySelectColumns = `
	m.id,
	m.employment_relationship_id,
	g.id,
	g.code,
	g.name,
	v.id,
	v.version_no,
	v.status,
	v.row_version,
	v.effective_from,
	v.effective_to,
	m.effective_from,
	m.effective_to,
	v.minutes_per_credit,
	v.work_unit_minutes,
	v.rounding_policy,
	v.early_arrival_strategy,
	v.late_arrival_grace_minutes,
	v.early_ot_enabled,
	v.early_ot_qualification_grace_minutes,
	v.early_ot_max_minutes,
	v.break_auto_deduct,
	v.break_is_overtime,
	v.ot_aggregation_mode,
	v.working_pattern_id,
	v.pay_method,
	v.monthly_denominator,
	v.monthly_standard_days::text,
	v.rate_component_definition_id,
	v.paid_leave_credit_mode,
	ov.id,
	ov.policy_id,
	ov.version_no,
	ov.daily_cap_minutes,
	ov.monthly_cap_hours::text,
	ov.yearly_cap_hours::text,
	ov.rules
`

func normalizeDateUTC(t time.Time) time.Time {
	return time.Date(t.Year(), t.Month(), t.Day(), 0, 0, 0, 0, time.UTC)
}

func normalizeOptionalDateUTC(t *time.Time) *time.Time {
	if t == nil {
		return nil
	}
	v := normalizeDateUTC(*t)
	return &v
}

func hydrateResolvedPolicy(p *workpolicy.ResolvedPolicy, status, payMethod, paidLeaveMode string, denominator, monthlyDays, monthlyCap, yearlyCap *string, rulesRaw []byte) error {
	p.Status = workpolicy.VersionStatus(status)
	p.PayMethod = workpolicy.PayMethod(payMethod)
	p.PaidLeaveCreditMode = workpolicy.PaidLeaveCreditMode(paidLeaveMode)
	p.EffectiveFrom = normalizeDateUTC(p.EffectiveFrom)
	p.EffectiveTo = normalizeOptionalDateUTC(p.EffectiveTo)
	p.MembershipEffectiveFrom = normalizeDateUTC(p.MembershipEffectiveFrom)
	p.MembershipEffectiveTo = normalizeOptionalDateUTC(p.MembershipEffectiveTo)
	if denominator != nil && *denominator != "" {
		v := workpolicy.MonthlyDenominator(*denominator)
		p.MonthlyDenominator = &v
	}
	var err error
	if p.MonthlyStandardDays, err = parseOptionalDecimal(monthlyDays); err != nil {
		return fmt.Errorf("work policy resolver: invalid monthly_standard_days: %w", err)
	}
	if p.MonthlyCapHours, err = parseOptionalDecimal(monthlyCap); err != nil {
		return fmt.Errorf("work policy resolver: invalid monthly_cap_hours: %w", err)
	}
	if p.YearlyCapHours, err = parseOptionalDecimal(yearlyCap); err != nil {
		return fmt.Errorf("work policy resolver: invalid yearly_cap_hours: %w", err)
	}
	p.OvertimeRules = make([]workpolicy.OvertimeRule, 0)
	if len(rulesRaw) > 0 {
		if err := json.Unmarshal(rulesRaw, &p.OvertimeRules); err != nil {
			return fmt.Errorf("work policy resolver: invalid overtime rules json: %w", err)
		}
	}
	return nil
}

func scanResolvedPolicy(scan func(...any) error) (workpolicy.ResolvedPolicy, error) {
	var (
		p                                             workpolicy.ResolvedPolicy
		status, payMethod, paidLeaveMode              string
		denominator, monthlyDays, monthlyCap, yearCap *string
		rulesRaw                                      []byte
	)
	if err := scan(
		&p.MembershipID, &p.EmploymentRelationshipID, &p.GroupID, &p.GroupCode, &p.GroupName,
		&p.VersionID, &p.VersionNo, &status, &p.RowVersion, &p.EffectiveFrom, &p.EffectiveTo,
		&p.MembershipEffectiveFrom, &p.MembershipEffectiveTo,
		&p.MinutesPerCredit, &p.WorkUnitMinutes, &p.RoundingPolicy, &p.EarlyArrivalStrategy,
		&p.LateArrivalGraceMinutes, &p.EarlyOTEnabled, &p.EarlyOTQualificationGraceMinutes,
		&p.EarlyOTMaxMinutes, &p.BreakAutoDeduct, &p.BreakIsOvertime, &p.OTAggregationMode,
		&p.WorkingPatternID, &payMethod, &denominator, &monthlyDays, &p.RateComponentDefinitionID,
		&paidLeaveMode, &p.OvertimePolicyVersionID, &p.OvertimePolicyID, &p.OvertimePolicyVersionNo,
		&p.DailyCapMinutes, &monthlyCap, &yearCap, &rulesRaw,
	); err != nil {
		return workpolicy.ResolvedPolicy{}, err
	}
	if err := hydrateResolvedPolicy(&p, status, payMethod, paidLeaveMode, denominator, monthlyDays, monthlyCap, yearCap, rulesRaw); err != nil {
		return workpolicy.ResolvedPolicy{}, err
	}
	return p, nil
}

func (WorkPolicyResolver) ResolveAt(ctx context.Context, db DBTX, relationshipID uuid.UUID, asOf time.Time) (workpolicy.ResolvedPolicy, error) {
	if db == nil {
		return workpolicy.ResolvedPolicy{}, errors.New("work policy resolver: database is not configured")
	}
	asOfStr := asOf.Format("2006-01-02")
	row := db.QueryRow(ctx, `
		SELECT `+resolvedPolicySelectColumns+`
		FROM work_policy_memberships m
		JOIN employment_relationships er ON er.id = m.employment_relationship_id
		JOIN work_policy_groups g ON g.id = m.group_id
		JOIN work_policy_versions v ON v.group_id = m.group_id
		JOIN overtime_policy_versions ov ON ov.id = v.overtime_policy_version_id
		WHERE m.employment_relationship_id = $1
		  AND er.deleted_at IS NULL
		  AND er.start_date <= $2::date AND (er.end_date IS NULL OR er.end_date >= $2::date)
		  AND m.effective_from <= $2::date AND (m.effective_to IS NULL OR m.effective_to > $2::date)
		  AND v.status = 'published'
		  AND v.effective_from <= $2::date AND (v.effective_to IS NULL OR v.effective_to > $2::date)
		ORDER BY v.effective_from DESC, v.version_no DESC, v.id DESC
		LIMIT 1`, relationshipID, asOfStr)
	resolved, err := scanResolvedPolicy(row.Scan)
	if errors.Is(err, pgx.ErrNoRows) {
		return workpolicy.ResolvedPolicy{}, api.ErrWorkPolicyResolutionNotFound
	}
	return resolved, err
}

func (WorkPolicyResolver) ResolveRange(ctx context.Context, db DBTX, relationshipIDs []uuid.UUID, start, end time.Time) (map[uuid.UUID][]workpolicy.ResolvedPolicy, error) {
	result := make(map[uuid.UUID][]workpolicy.ResolvedPolicy, len(relationshipIDs))
	if len(relationshipIDs) == 0 {
		return result, nil
	}
	if db == nil {
		return nil, errors.New("work policy resolver: database is not configured")
	}
	rows, err := db.Query(ctx, `
		SELECT `+resolvedPolicySelectColumns+`
		FROM work_policy_memberships m
		JOIN employment_relationships er ON er.id = m.employment_relationship_id
		JOIN work_policy_groups g ON g.id = m.group_id
		JOIN work_policy_versions v ON v.group_id = m.group_id
		JOIN overtime_policy_versions ov ON ov.id = v.overtime_policy_version_id
		WHERE m.employment_relationship_id = ANY($1::uuid[])
		  AND er.deleted_at IS NULL
		  AND er.start_date < $3::date AND (er.end_date IS NULL OR er.end_date >= $2::date)
		  AND m.effective_from < $3::date AND (m.effective_to IS NULL OR m.effective_to > $2::date)
		  AND v.status = 'published'
		  AND v.effective_from < $3::date AND (v.effective_to IS NULL OR v.effective_to > $2::date)
		  AND GREATEST(m.effective_from, v.effective_from) < COALESCE(LEAST(m.effective_to, v.effective_to), 'infinity'::date)
		ORDER BY m.employment_relationship_id, GREATEST(m.effective_from, v.effective_from) ASC, v.version_no DESC, v.id DESC`,
		relationshipIDs, start.Format("2006-01-02"), end.AddDate(0, 0, 1).Format("2006-01-02"))
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	for rows.Next() {
		item, scanErr := scanResolvedPolicy(rows.Scan)
		if scanErr != nil {
			return nil, scanErr
		}
		result[item.EmploymentRelationshipID] = append(result[item.EmploymentRelationshipID], item)
	}
	return result, rows.Err()
}

func (r WorkPolicyResolver) ResolveForEmploymentAt(ctx context.Context, db DBTX, employmentID uuid.UUID, asOf time.Time) (workpolicy.ResolvedPolicy, error) {
	if db == nil {
		return workpolicy.ResolvedPolicy{}, errors.New("work policy resolver: database is not configured")
	}
	var relationshipID uuid.UUID
	err := db.QueryRow(ctx, `SELECT relationship_id FROM employments WHERE id = $1 AND deleted_at IS NULL`, employmentID).Scan(&relationshipID)
	if errors.Is(err, pgx.ErrNoRows) {
		return workpolicy.ResolvedPolicy{}, api.ErrWorkPolicyResolutionNotFound
	}
	if err != nil {
		return workpolicy.ResolvedPolicy{}, err
	}
	return r.ResolveAt(ctx, db, relationshipID, asOf)
}

func (WorkPolicyResolver) ResolveRangeForEmployments(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, start, end time.Time) (map[uuid.UUID][]workpolicy.ResolvedPolicy, error) {
	result := make(map[uuid.UUID][]workpolicy.ResolvedPolicy, len(employmentIDs))
	if len(employmentIDs) == 0 {
		return result, nil
	}
	if db == nil {
		return nil, errors.New("work policy resolver: database is not configured")
	}
	rows, err := db.Query(ctx, `
		SELECT e.id, `+resolvedPolicySelectColumns+`
		FROM employments e
		JOIN employment_relationships er ON er.id = e.relationship_id AND er.deleted_at IS NULL
		JOIN work_policy_memberships m ON m.employment_relationship_id = e.relationship_id
		JOIN work_policy_groups g ON g.id = m.group_id
		JOIN work_policy_versions v ON v.group_id = m.group_id
		JOIN overtime_policy_versions ov ON ov.id = v.overtime_policy_version_id
		WHERE e.id = ANY($1::uuid[])
		  AND e.deleted_at IS NULL
		  AND er.start_date < $3::date AND (er.end_date IS NULL OR er.end_date >= $2::date)
		  AND m.effective_from < $3::date AND (m.effective_to IS NULL OR m.effective_to > $2::date)
		  AND v.status = 'published'
		  AND v.effective_from < $3::date AND (v.effective_to IS NULL OR v.effective_to > $2::date)
		  AND GREATEST(m.effective_from, v.effective_from) < COALESCE(LEAST(m.effective_to, v.effective_to), 'infinity'::date)
		ORDER BY e.id, GREATEST(m.effective_from, v.effective_from) ASC, v.version_no DESC, v.id DESC`,
		employmentIDs, start.Format("2006-01-02"), end.AddDate(0, 0, 1).Format("2006-01-02"))
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	for rows.Next() {
		var empID uuid.UUID
		item, scanErr := scanResolvedPolicy(func(dest ...any) error {
			all := make([]any, 0, len(dest)+1)
			all = append(all, &empID)
			all = append(all, dest...)
			return rows.Scan(all...)
		})
		if scanErr != nil {
			return nil, scanErr
		}
		result[empID] = append(result[empID], item)
	}
	return result, rows.Err()
}

func (WorkPolicyResolver) LoadResolvedVersion(ctx context.Context, db DBTX, groupID, versionID uuid.UUID) (workpolicy.ResolvedPolicy, error) {
	if db == nil {
		return workpolicy.ResolvedPolicy{}, errors.New("work policy resolver: database is not configured")
	}
	row := db.QueryRow(ctx, `
		SELECT
			'00000000-0000-0000-0000-000000000000'::uuid,
			'00000000-0000-0000-0000-000000000000'::uuid,
			g.id, g.code, g.name, v.id, v.version_no, v.status, v.row_version,
			v.effective_from, v.effective_to, v.effective_from, v.effective_to,
			v.minutes_per_credit, v.work_unit_minutes, v.rounding_policy, v.early_arrival_strategy,
			v.late_arrival_grace_minutes, v.early_ot_enabled, v.early_ot_qualification_grace_minutes,
			v.early_ot_max_minutes, v.break_auto_deduct, v.break_is_overtime, v.ot_aggregation_mode,
			v.working_pattern_id, v.pay_method, v.monthly_denominator, v.monthly_standard_days::text,
			v.rate_component_definition_id, v.paid_leave_credit_mode,
			ov.id, ov.policy_id, ov.version_no, ov.daily_cap_minutes, ov.monthly_cap_hours::text, ov.yearly_cap_hours::text, ov.rules
		FROM work_policy_versions v
		JOIN work_policy_groups g ON g.id = v.group_id
		JOIN overtime_policy_versions ov ON ov.id = v.overtime_policy_version_id
		WHERE v.group_id = $1 AND v.id = $2`, groupID, versionID)
	resolved, err := scanResolvedPolicy(row.Scan)
	if errors.Is(err, pgx.ErrNoRows) {
		return workpolicy.ResolvedPolicy{}, api.ErrWorkPolicyGroupNotFound
	}
	return resolved, err
}

func FindResolvedPolicyForDate(policies []workpolicy.ResolvedPolicy, day time.Time) (workpolicy.ResolvedPolicy, bool) {
	for i := len(policies) - 1; i >= 0; i-- {
		if policies[i].CoversDate(day) {
			return policies[i], true
		}
	}
	return workpolicy.ResolvedPolicy{}, false
}
