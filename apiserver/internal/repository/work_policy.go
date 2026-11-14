package repository

import (
	"context"
	"errors"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type WorkPolicyRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r WorkPolicyRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("work policy repository: database is not configured")
	}
	return r.DB, nil
}

func (r WorkPolicyRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("work policy repository: database does not support transactions")
	}
	return beginner, nil
}

func (r WorkPolicyRepository) ListWorkPolicyGroupVersions(ctx context.Context, page, limit int) (api.WorkPolicyGroupListResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.WorkPolicyGroupListResponse{}, err
	}
	offset := (page - 1) * limit
	rows, err := db.Query(ctx, `
		SELECT v.id, v.group_id, g.name, v.work_credit_policy_id, wc.name, wc.minutes_per_credit,
		       wc.attendance_policy_id, wc.working_pattern_id, v.pay_basis_policy_id, pb.name,
		       pb.method, pb.monthly_denominator, pb.monthly_standard_days::text, v.overtime_policy_id,
		       v.effective_from::text, v.effective_to::text
		FROM labor_policy_group_versions v
		JOIN labor_policy_groups g ON g.id = v.group_id
		JOIN work_credit_policies wc ON wc.id = v.work_credit_policy_id
		JOIN pay_basis_policies pb ON pb.id = v.pay_basis_policy_id
		ORDER BY v.effective_from DESC, v.id DESC
		LIMIT $1 OFFSET $2`, limit, offset)
	if err != nil {
		return api.WorkPolicyGroupListResponse{}, err
	}
	defer rows.Close()
	result := api.WorkPolicyGroupListResponse{Data: make([]api.WorkPolicyGroupVersion, 0), Page: page, Limit: limit}
	for rows.Next() {
		item, err := workPolicyGroupVersion(rows)
		if err != nil {
			return api.WorkPolicyGroupListResponse{}, err
		}
		result.Data = append(result.Data, item)
	}
	if err := rows.Err(); err != nil {
		return api.WorkPolicyGroupListResponse{}, err
	}
	rows.Close()
	if err := db.QueryRow(ctx, `SELECT count(*) FROM labor_policy_group_versions`).Scan(&result.Total); err != nil {
		return api.WorkPolicyGroupListResponse{}, err
	}
	return result, nil
}

func (r WorkPolicyRepository) CreateWorkPolicyGroup(ctx context.Context, req api.CreateWorkPolicyGroupRequest, actorID uuid.UUID) (api.WorkPolicyGroupVersion, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.WorkPolicyGroupVersion{}, err
	}
	var referencesExist bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1 FROM attendance_policies WHERE id = $1
	) AND EXISTS (
		SELECT 1 FROM working_patterns WHERE id = $2 AND is_active = TRUE
	) AND EXISTS (
		SELECT 1 FROM overtime_policy_definitions WHERE id = $3
	)`, req.AttendancePolicyID, req.WorkingPatternID, req.OvertimePolicyID).Scan(&referencesExist); err != nil {
		return api.WorkPolicyGroupVersion{}, err
	}
	if !referencesExist {
		return api.WorkPolicyGroupVersion{}, api.ErrWorkPolicyReferenceNotFound
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.WorkPolicyGroupVersion{}, err
	}
	from, err := time.Parse("2006-01-02", strings.TrimSpace(req.EffectiveFrom))
	if err != nil {
		return api.WorkPolicyGroupVersion{}, api.ErrWorkPolicyGroupValidation
	}
	var to *time.Time
	if req.EffectiveTo != nil && strings.TrimSpace(*req.EffectiveTo) != "" {
		parsed, parseErr := time.Parse("2006-01-02", strings.TrimSpace(*req.EffectiveTo))
		if parseErr != nil {
			return api.WorkPolicyGroupVersion{}, api.ErrWorkPolicyGroupValidation
		}
		to = &parsed
	}
	groupID, workCreditID, payBasisID, versionID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	result := workPolicyGroupVersionResult(req, groupID, workCreditID, payBasisID, versionID)
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		return insertWorkPolicyGroup(ctx, tx, req, actorID, from, to, groupID, workCreditID, payBasisID, versionID)
	})
	if err != nil {
		return api.WorkPolicyGroupVersion{}, err
	}
	return result, nil
}

func (r WorkPolicyRepository) CreateWorkPolicyGroupVersion(ctx context.Context, groupID uuid.UUID, req api.CreateWorkPolicyGroupVersionRequest, actorID uuid.UUID) (api.WorkPolicyGroupVersion, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.WorkPolicyGroupVersion{}, err
	}
	if err := validateVersionGroup(ctx, db, groupID, req); err != nil {
		return api.WorkPolicyGroupVersion{}, err
	}
	from, to, err := parseWorkPolicyRange(req.EffectiveFrom, req.EffectiveTo)
	if err != nil {
		return api.WorkPolicyGroupVersion{}, api.ErrWorkPolicyGroupValidation
	}
	workCreditID, payBasisID, versionID := uuid.New(), uuid.New(), uuid.New()
	result := workPolicyGroupVersionResult(api.CreateWorkPolicyGroupRequest(req), groupID, workCreditID, payBasisID, versionID)
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.WorkPolicyGroupVersion{}, err
	}
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		return insertWorkPolicyVersion(ctx, tx, req, actorID, from, to, groupID, workCreditID, payBasisID, versionID)
	})
	if err != nil {
		return api.WorkPolicyGroupVersion{}, err
	}
	return result, nil
}

func validateVersionGroup(ctx context.Context, db DBTX, groupID uuid.UUID, req api.CreateWorkPolicyGroupVersionRequest) error {
	var groupName string
	if err := db.QueryRow(ctx, `SELECT name FROM labor_policy_groups WHERE id = $1`, groupID).Scan(&groupName); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrWorkPolicyGroupNotFound
		}
		return err
	}
	if strings.TrimSpace(groupName) != strings.TrimSpace(req.Name) {
		return api.ErrWorkPolicyGroupValidation
	}
	var referencesExist bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM attendance_policies WHERE id = $1) AND EXISTS (SELECT 1 FROM working_patterns WHERE id = $2 AND is_active = TRUE) AND EXISTS (SELECT 1 FROM overtime_policy_definitions WHERE id = $3)`, req.AttendancePolicyID, req.WorkingPatternID, req.OvertimePolicyID).Scan(&referencesExist); err != nil {
		return err
	}
	if !referencesExist {
		return api.ErrWorkPolicyReferenceNotFound
	}
	return nil
}

func insertWorkPolicyVersion(ctx context.Context, tx pgx.Tx, req api.CreateWorkPolicyGroupVersionRequest, actorID uuid.UUID, from, to *time.Time, groupID, workCreditID, payBasisID, versionID uuid.UUID) error {
	if err := insertPolicyDefinitions(ctx, tx, req.Name, req.MinutesPerCredit, req.AttendancePolicyID, req.WorkingPatternID, req.PayMethod, req.MonthlyDenominator, req.MonthlyStandardDays, actorID, workCreditID, payBasisID); err != nil {
		return err
	}
	_, err := tx.Exec(ctx, `INSERT INTO labor_policy_group_versions (id, group_id, work_credit_policy_id, pay_basis_policy_id, overtime_policy_id, effective_from, effective_to, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`, versionID, groupID, workCreditID, payBasisID, req.OvertimePolicyID, from, to, actorID)
	if isConflictError(err) {
		return api.ErrConflict
	}
	return err
}

func insertPolicyDefinitions(ctx context.Context, tx pgx.Tx, name string, minutes int, attendancePolicyID, workingPatternID uuid.UUID, method workpolicy.PayMethod, denominator *workpolicy.MonthlyDenominator, days *decimal.Decimal, actorID, workCreditID, payBasisID uuid.UUID) error {
	if _, err := tx.Exec(ctx, `INSERT INTO work_credit_policies (id, name, minutes_per_credit, attendance_policy_id, working_pattern_id, created_by) VALUES ($1, $2, $3, $4, $5, $6)`, workCreditID, name, minutes, attendancePolicyID, workingPatternID, actorID); err != nil {
		return mapConstraintError(err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO pay_basis_policies (id, name, method, monthly_denominator, monthly_standard_days, created_by) VALUES ($1, $2, $3, $4, $5, $6)`, payBasisID, name, method, denominator, days, actorID); err != nil {
		return mapConstraintError(err)
	}
	return nil
}

func workPolicyGroupVersion(row interface{ Scan(...any) error }) (api.WorkPolicyGroupVersion, error) {
	var item api.WorkPolicyGroupVersion
	var method string
	var denominator *string
	var days *string
	if err := row.Scan(&item.ID, &item.GroupID, &item.Name, &item.WorkCreditPolicyID, &item.WorkCreditPolicyName, &item.MinutesPerCredit, &item.AttendancePolicyID, &item.WorkingPatternID, &item.PayBasisPolicyID, &item.PayBasisPolicyName, &method, &denominator, &days, &item.OvertimePolicyID, &item.EffectiveFrom, &item.EffectiveTo); err != nil {
		return api.WorkPolicyGroupVersion{}, err
	}
	item.PayMethod = apiWorkPolicyMethod(method)
	if denominator != nil && *denominator != "" {
		value := apiWorkPolicyDenominator(*denominator)
		item.MonthlyDenominator = &value
	}
	if days != nil {
		value, err := decimal.NewFromString(*days)
		if err != nil {
			return api.WorkPolicyGroupVersion{}, err
		}
		item.MonthlyStandardDays = &value
	}
	return item, nil
}

func workPolicyGroupVersionResult(req api.CreateWorkPolicyGroupRequest, groupID, workCreditID, payBasisID, versionID uuid.UUID) api.WorkPolicyGroupVersion {
	return api.WorkPolicyGroupVersion{
		ID: versionID, GroupID: groupID, Name: req.Name,
		WorkCreditPolicyID: workCreditID, WorkCreditPolicyName: req.Name,
		MinutesPerCredit: req.MinutesPerCredit, AttendancePolicyID: req.AttendancePolicyID,
		WorkingPatternID: req.WorkingPatternID, PayBasisPolicyID: payBasisID,
		PayBasisPolicyName: req.Name, PayMethod: req.PayMethod,
		MonthlyDenominator: req.MonthlyDenominator, MonthlyStandardDays: req.MonthlyStandardDays,
		OvertimePolicyID: req.OvertimePolicyID, EffectiveFrom: req.EffectiveFrom, EffectiveTo: req.EffectiveTo,
	}
}

func insertWorkPolicyGroup(ctx context.Context, tx pgx.Tx, req api.CreateWorkPolicyGroupRequest, actorID uuid.UUID, from time.Time, to *time.Time, groupID, workCreditID, payBasisID, versionID uuid.UUID) error {
	if _, err := tx.Exec(ctx, `INSERT INTO work_credit_policies (id, name, minutes_per_credit, attendance_policy_id, working_pattern_id, created_by) VALUES ($1, $2, $3, $4, $5, $6)`, workCreditID, req.Name, req.MinutesPerCredit, req.AttendancePolicyID, req.WorkingPatternID, actorID); err != nil {
		return mapConstraintError(err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO pay_basis_policies (id, name, method, monthly_denominator, monthly_standard_days, created_by) VALUES ($1, $2, $3, $4, $5, $6)`, payBasisID, req.Name, req.PayMethod, req.MonthlyDenominator, req.MonthlyStandardDays, actorID); err != nil {
		return mapConstraintError(err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO labor_policy_groups (id, name, created_by) VALUES ($1, $2, $3)`, groupID, req.Name, actorID); err != nil {
		return mapConstraintError(err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO labor_policy_group_versions (id, group_id, work_credit_policy_id, pay_basis_policy_id, overtime_policy_id, effective_from, effective_to, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`, versionID, groupID, workCreditID, payBasisID, req.OvertimePolicyID, from, to, actorID); err != nil {
		return mapConstraintError(err)
	}
	return nil
}

func (r WorkPolicyRepository) ListWorkPolicyMemberships(ctx context.Context, groupID uuid.UUID) ([]api.WorkPolicyMembership, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, employment_id, group_id, effective_from::text, effective_to::text FROM labor_policy_memberships WHERE group_id = $1 ORDER BY effective_from DESC, id DESC`, groupID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.WorkPolicyMembership, 0)
	for rows.Next() {
		var item api.WorkPolicyMembership
		if err := rows.Scan(&item.ID, &item.EmploymentID, &item.GroupID, &item.EffectiveFrom, &item.EffectiveTo); err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (r WorkPolicyRepository) CreateWorkPolicyMembership(ctx context.Context, groupID uuid.UUID, req api.CreateWorkPolicyMembershipRequest, actorID uuid.UUID) (api.WorkPolicyMembership, error) {
	from, to, err := parseWorkPolicyRange(req.EffectiveFrom, req.EffectiveTo)
	if err != nil {
		return api.WorkPolicyMembership{}, api.ErrWorkPolicyGroupValidation
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.WorkPolicyMembership{}, err
	}
	item := api.WorkPolicyMembership{ID: uuid.New(), EmploymentID: req.EmploymentID, GroupID: groupID, EffectiveFrom: req.EffectiveFrom, EffectiveTo: req.EffectiveTo}
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var exists bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM labor_policy_groups WHERE id = $1)`, groupID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrWorkPolicyGroupNotFound
		}
		if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND deleted_at IS NULL)`, req.EmploymentID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrEmploymentNotFound
		}
		_, err := tx.Exec(ctx, `INSERT INTO labor_policy_memberships (id, employment_id, group_id, effective_from, effective_to, created_by) VALUES ($1, $2, $3, $4, $5, $6)`, item.ID, req.EmploymentID, groupID, from, to, actorID)
		if isConflictError(err) {
			return api.ErrWorkPolicyMembershipConflict
		}
		return err
	})
	if err != nil {
		return api.WorkPolicyMembership{}, err
	}
	return item, nil
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
	return &from, &to, nil
}

func apiWorkPolicyMethod(value string) workpolicy.PayMethod {
	return workpolicy.PayMethod(value)
}

func apiWorkPolicyDenominator(value string) workpolicy.MonthlyDenominator {
	return workpolicy.MonthlyDenominator(value)
}
