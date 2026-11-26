package repository

import (
	"context"
	"errors"
	"fmt"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"strings"
	"time"
)

func loadIncentiveTiersMap(ctx context.Context, db DBTX, policyIDs []uuid.UUID) (map[uuid.UUID][]api.IncentiveTier, error) {
	tierRows, err := db.Query(ctx, `SELECT id, policy_id, priority, payout_amount::text, rule_config
FROM payroll_incentive_tiers
WHERE policy_id = ANY($1)
ORDER BY priority ASC`, policyIDs)
	if err != nil {
		return nil, err
	}
	defer tierRows.Close()

	tiersMap := make(map[uuid.UUID][]api.IncentiveTier)
	for tierRows.Next() {
		var t api.IncentiveTier
		var payoutStr string
		if err := tierRows.Scan(&t.ID, &t.PolicyID, &t.Priority, &payoutStr, &t.RuleConfig); err != nil {
			return nil, err
		}
		t.PayoutAmount, _ = decimal.NewFromString(payoutStr)
		tiersMap[t.PolicyID] = append(tiersMap[t.PolicyID], t)
	}
	return tiersMap, tierRows.Err()
}

func (r PayrollRepository) GetIncentivePolicies(ctx context.Context) ([]api.PayrollIncentivePolicyResponse, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, name, base_amount::text, payout_type, is_active, created_at, effective_from::text, effective_to::text
FROM payroll_incentive_policies
WHERE is_active = true AND deleted_at IS NULL
ORDER BY created_at ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var policies []api.PayrollIncentivePolicy
	var policyIDs []uuid.UUID
	for rows.Next() {
		var p api.PayrollIncentivePolicy
		var baseStr string
		var created time.Time
		var effTo *string
		if err := rows.Scan(&p.ID, &p.Name, &baseStr, &p.PayoutType, &p.IsActive, &created, &p.EffectiveFrom, &effTo); err != nil {
			return nil, err
		}
		p.BaseAmount, _ = decimal.NewFromString(baseStr)
		p.CreatedAt = created.UTC().Format(time.RFC3339)
		p.EffectiveTo = effTo
		policies = append(policies, p)
		policyIDs = append(policyIDs, p.ID)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	if len(policies) == 0 {
		return []api.PayrollIncentivePolicyResponse{}, nil
	}

	tiersMap, err := loadIncentiveTiersMap(ctx, db, policyIDs)
	if err != nil {
		return nil, err
	}

	result := make([]api.PayrollIncentivePolicyResponse, 0, len(policies))
	for _, p := range policies {
		tiers := tiersMap[p.ID]
		if tiers == nil {
			tiers = []api.IncentiveTier{}
		}
		result = append(result, api.PayrollIncentivePolicyResponse{
			PayrollIncentivePolicy: p,
			Tiers:                  tiers,
		})
	}
	return result, nil
}

func (r PayrollRepository) CreateIncentivePolicy(ctx context.Context, req api.CreatePayrollIncentivePolicyRequest) (api.PayrollIncentivePolicy, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}
	policyID := uuid.New()
	var createdAt time.Time
	payoutType := req.PayoutType
	if payoutType == "" {
		payoutType = "fixed"
	}
	err = db.QueryRow(ctx, `INSERT INTO payroll_incentive_policies (id, name, base_amount, payout_type, is_active, effective_from, effective_to, created_at, deleted_at)
VALUES ($1, $2, $3, $4, true, $5::date, NULL, NOW(), NULL)
RETURNING created_at`, policyID, strings.TrimSpace(req.Name), req.BaseAmount.InexactFloat64(), payoutType, req.EffectiveFrom).Scan(&createdAt)
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}

	for _, tier := range req.Tiers {
		tierID := uuid.New()
		ruleCfg := tier.RuleConfig
		if len(ruleCfg) == 0 {
			ruleCfg = []byte("{}")
		}
		_, err = db.Exec(ctx, `INSERT INTO payroll_incentive_tiers (id, policy_id, priority, payout_amount, rule_config)
VALUES ($1, $2, $3, $4, $5)`, tierID, policyID, tier.Priority, tier.PayoutAmount.InexactFloat64(), ruleCfg)
		if err != nil {
			return api.PayrollIncentivePolicy{}, err
		}
	}

	return api.PayrollIncentivePolicy{
		ID:            policyID,
		Name:          strings.TrimSpace(req.Name),
		BaseAmount:    req.BaseAmount,
		PayoutType:    payoutType,
		IsActive:      true,
		CreatedAt:     createdAt.UTC().Format(time.RFC3339),
		EffectiveFrom: req.EffectiveFrom,
		EffectiveTo:   nil,
	}, nil
}

func (r PayrollRepository) DeleteIncentivePolicy(ctx context.Context, policyID ids.PayrollIncentivePolicyID) error {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE payroll_incentive_policies SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL`, uuid.UUID(policyID))
	return err
}

type incentiveTierClone struct {
	priority int
	payout   decimal.Decimal
	config   []byte
}

func loadPolicyTiersToClone(ctx context.Context, db DBTX, policyID uuid.UUID) ([]incentiveTierClone, error) {
	tierRows, err := db.Query(ctx, `SELECT priority, payout_amount::text, rule_config
FROM payroll_incentive_tiers
WHERE policy_id = $1
ORDER BY priority ASC`, policyID)
	if err != nil {
		return nil, err
	}
	defer tierRows.Close()

	var origTiers []incentiveTierClone
	for tierRows.Next() {
		var priority int
		var payoutStr string
		var cfg []byte
		if err := tierRows.Scan(&priority, &payoutStr, &cfg); err != nil {
			return nil, err
		}
		payout, _ := decimal.NewFromString(payoutStr)
		origTiers = append(origTiers, incentiveTierClone{priority: priority, payout: payout, config: cfg})
	}
	return origTiers, tierRows.Err()
}

func (r PayrollRepository) CloneIncentivePolicy(ctx context.Context, policyID ids.PayrollIncentivePolicyID) (api.PayrollIncentivePolicy, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}
	var origName, origPayout, origBaseStr string
	err = db.QueryRow(ctx, `SELECT name, payout_type, base_amount::text
FROM payroll_incentive_policies
WHERE id = $1 AND deleted_at IS NULL`, uuid.UUID(policyID)).Scan(&origName, &origPayout, &origBaseStr)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.PayrollIncentivePolicy{}, api.ErrIncentivePolicyNotFound
	}
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}
	origBase, _ := decimal.NewFromString(origBaseStr)

	origTiers, err := loadPolicyTiersToClone(ctx, db, uuid.UUID(policyID))
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}

	newPolicyID := uuid.New()
	newName := fmt.Sprintf("%s (Copy)", origName)
	var createdAt time.Time
	var effFrom string
	err = db.QueryRow(ctx, `INSERT INTO payroll_incentive_policies (id, name, base_amount, payout_type, is_active, effective_from, effective_to, created_at, deleted_at)
VALUES ($1, $2, $3, $4, false, CURRENT_DATE, NULL, NOW(), NULL)
RETURNING created_at, effective_from::text`, newPolicyID, newName, origBase.InexactFloat64(), origPayout).Scan(&createdAt, &effFrom)
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}

	for _, t := range origTiers {
		cfg := t.config
		if len(cfg) == 0 {
			cfg = []byte("{}")
		}
		if _, err = db.Exec(ctx, `INSERT INTO payroll_incentive_tiers (id, policy_id, priority, payout_amount, rule_config)
VALUES ($1, $2, $3, $4, $5)`, uuid.New(), newPolicyID, t.priority, t.payout.InexactFloat64(), cfg); err != nil {
			return api.PayrollIncentivePolicy{}, err
		}
	}

	return api.PayrollIncentivePolicy{
		ID:            newPolicyID,
		Name:          newName,
		BaseAmount:    origBase,
		PayoutType:    origPayout,
		IsActive:      false,
		CreatedAt:     createdAt.UTC().Format(time.RFC3339),
		EffectiveFrom: effFrom,
		EffectiveTo:   nil,
	}, nil
}
