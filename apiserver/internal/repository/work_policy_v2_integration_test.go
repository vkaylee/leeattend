//go:build integration

package repository

import (
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

func TestIntegrationWorkPolicyV2_LifecycleAtomicTransferAndBatchSimulation(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)

	actorID, employerID := uuid.New(), uuid.New()
	emp1ID, employment1ID := uuid.New(), uuid.New()
	emp2ID, employment2ID := uuid.New(), uuid.New()
	compDefID := uuid.New()

	mustExec := func(q string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, q, args...); err != nil {
			t.Fatalf("exec %q: %v", q, err)
		}
	}
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'wpv2-employer')`, employerID)
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, emp1ID, employerID, employment1ID, "WPV2-1-"+emp1ID.String()[:6], "Worker One", "2026-01-01", "2026-01-01", "")
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, emp2ID, employerID, employment2ID, "WPV2-2-"+emp2ID.String()[:6], "Worker Two", "2026-01-01", "2026-01-01", "")
	var rel1ID, rel2ID uuid.UUID
	_ = pool.QueryRow(ctx, `SELECT relationship_id FROM employments WHERE id = $1`, employment1ID).Scan(&rel1ID)
	_ = pool.QueryRow(ctx, `SELECT relationship_id FROM employments WHERE id = $1`, employment2ID).Scan(&rel2ID)

	mustExec(`INSERT INTO salary_component_definitions (id, name, code, component_type) VALUES ($1, 'Base', $2, 'base_salary')`, compDefID, "WPV2_BASE_"+compDefID.String()[:8])
	mustExec(`INSERT INTO employee_salary_components (id, component_definition_id, amount, effective_date, employment_id) VALUES (gen_random_uuid(), $1, 22000000, DATE '2026-01-01', $2)`, compDefID, employment1ID)
	mustExec(`INSERT INTO attendance_daily_summaries (id, employment_id, date, actual_work_minutes, standard_minutes, net_paid_minutes, approved_ot_minutes, ot_status) VALUES (gen_random_uuid(), $1, DATE '2026-03-02', 480, 480, 480, 120, 'approved')`, employment1ID)

	// 1. Create & publish immutable OvertimePolicyVersion
	otRepo := OvertimeRepository{DB: pool}
	dailyCap := 240
	monthlyCap := decimal.NewFromInt(40)
	otDetail, err := otRepo.CreateOvertimePolicy(ctx, api.CreateOvertimePolicyRequest{
		Name: "WPV2 OT", DailyCapMinutes: &dailyCap, MonthlyCapHours: &monthlyCap,
		Rules: []workpolicy.OvertimeRule{{Priority: 50, DayType: workpolicy.OvertimeDayWeekday, DaysOfWeek: 127, Multiplier: decimal.RequireFromString("1.50")}},
	}, actorID)
	if err != nil {
		t.Fatalf("create OT policy: %v", err)
	}
	otVer, err := otRepo.PublishOvertimePolicyVersion(ctx, otDetail.Policy.ID, otDetail.Versions[0].ID, api.PublishOvertimePolicyVersionRequest{ExpectedRowVersion: 1}, actorID)
	if err != nil || otVer.Status != workpolicy.StatusPublished {
		t.Fatalf("publish OT version: %+v err=%v", otVer, err)
	}
	// Editing published OT version must fail with conflict
	if _, err := otRepo.PatchOvertimePolicyVersion(ctx, otDetail.Policy.ID, otVer.ID, api.PatchOvertimePolicyVersionRequest{ExpectedRowVersion: otVer.RowVersion, Rules: otVer.Rules}); !errors.Is(err, api.ErrOvertimeConflict) {
		t.Fatalf("expected conflict editing published OT version, got %v", err)
	}

	// 2. Create Group A + draft v1 -> patch (verify 409 on stale row_version) -> publish -> assign rel1
	wpRepo := WorkPolicyRepository{DB: pool}
	denom := workpolicy.MonthlyDenominatorFixed
	days22 := decimal.NewFromInt(22)
	draftA1, err := wpRepo.CreateWorkPolicyGroup(ctx, workpolicy.CreateGroupInput{
		Code: "WPV2-A-" + uuid.NewString()[:6], Name: "Group A",
		Draft: workpolicy.VersionDraftFields{
			EffectiveFrom: "2026-01-01", MinutesPerCredit: 480, WorkUnitMinutes: 1, RoundingPolicy: "none",
			EarlyArrivalStrategy: "clamp", BreakAutoDeduct: true, OTAggregationMode: "aggregate",
			PayMethod: workpolicy.PayMethodMonthly, MonthlyDenominator: &denom, MonthlyStandardDays: &days22,
			PaidLeaveCreditMode: workpolicy.PaidLeaveCreditStandardDay, OvertimePolicyVersionID: otVer.ID,
		},
	}, ids.UserID(actorID))
	if err != nil || draftA1.Status != workpolicy.StatusDraft {
		t.Fatalf("create group A: %+v err=%v", draftA1, err)
	}
	if _, err := wpRepo.PatchWorkPolicyVersion(ctx, workpolicy.PatchVersionInput{
		GroupID: workpolicy.GroupID(draftA1.GroupID), VersionID: workpolicy.VersionID(draftA1.ID), ExpectedRowVersion: 99,
		Draft: draftFromVersion(draftA1),
	}); !errors.Is(err, api.ErrWorkPolicyVersionConflict) {
		t.Fatalf("expected stale row_version conflict, got %v", err)
	}
	pubA1, err := wpRepo.PublishWorkPolicyVersion(ctx, workpolicy.PublishVersionInput{
		GroupID: workpolicy.GroupID(draftA1.GroupID), VersionID: workpolicy.VersionID(draftA1.ID), ExpectedRowVersion: draftA1.RowVersion, IdempotencyKey: "wpv2-pub-a1", ActorID: ids.UserID(actorID),
	})
	if err != nil || pubA1.Status != workpolicy.StatusPublished {
		t.Fatalf("publish A1: %+v err=%v", pubA1, err)
	}
	if _, err := wpRepo.CreateWorkPolicyMembership(ctx, workpolicy.CreateMembershipInput{
		GroupID: workpolicy.GroupID(draftA1.GroupID), EmploymentRelationshipID: rel1ID, EffectiveFrom: "2026-01-01", ActorID: ids.UserID(actorID),
	}); err != nil {
		t.Fatalf("create membership rel1: %v", err)
	}

	// 3. Create draft v2 effective 2026-03-01 (changing fixed days 22 -> 20) & lock 2026-03 payroll period
	days20 := decimal.NewFromInt(20)
	draftA2Fields := draftFromVersion(pubA1)
	draftA2Fields.EffectiveFrom = "2026-03-01"
	draftA2Fields.MonthlyStandardDays = &days20
	draftA2, err := wpRepo.CreateWorkPolicyVersion(ctx, workpolicy.CreateVersionInput{
		GroupID: workpolicy.GroupID(draftA1.GroupID), Draft: &draftA2Fields,
	}, ids.UserID(actorID))
	if err != nil {
		t.Fatalf("create draft A2: %v", err)
	}
	mustExec(`INSERT INTO payroll_periods (employer_id, period_year, period_month, status, version, locked_by, locked_at) VALUES ($1, 2026, 3, 'locked', 1, $2, NOW()) ON CONFLICT (employer_id, period_year, period_month) DO UPDATE SET status = 'locked'`, employerID, actorID)

	// Publishing draft A2 touching locked 2026-03 must fail with ErrWorkPolicyLockedPeriod
	if _, err := wpRepo.PublishWorkPolicyVersion(ctx, workpolicy.PublishVersionInput{
		GroupID: workpolicy.GroupID(draftA1.GroupID), VersionID: workpolicy.VersionID(draftA2.ID), ExpectedRowVersion: draftA2.RowVersion, AutoClosePrevious: true, ActorID: ids.UserID(actorID),
	}); !errors.Is(err, api.ErrWorkPolicyLockedPeriod) {
		t.Fatalf("expected locked period conflict on publish, got %v", err)
	}

	// 4. Batch simulation MUST STILL WORK on locked period 2026-03 with ZERO DB writes!
	// rel1 has baseline+candidate (computed & affected); rel2 has NO policy (uncomputable, never coerced to zero!)
	var payslipCountBefore, runCountBefore int
	_ = pool.QueryRow(ctx, `SELECT count(*) FROM payslips WHERE employment_id = ANY($1::uuid[])`, []uuid.UUID{employment1ID, employment2ID}).Scan(&payslipCountBefore)
	_ = pool.QueryRow(ctx, `SELECT count(*) FROM payroll_calculation_runs WHERE employment_id = ANY($1::uuid[])`, []uuid.UUID{employment1ID, employment2ID}).Scan(&runCountBefore)

	payrollRepo := PayrollRepository{DB: pool}
	simResp, err := payrollRepo.SimulateWorkPolicyVersionBatch(ctx, draftA1.GroupID, draftA2.ID, api.WorkPolicyBatchSimulationRequest{
		Year: 2026, Month: 3, EmploymentRelationshipIDs: []uuid.UUID{rel1ID, rel2ID},
	})
	if err != nil {
		t.Fatalf("batch simulate on locked period failed: %v", err)
	}
	if simResp.Total != 2 || simResp.Computed != 1 || simResp.Affected != 1 || simResp.Uncomputable != 1 {
		t.Fatalf("unexpected batch sim summary: %+v", simResp)
	}
	if simResp.Items[0].Status != "computed" || !simResp.Items[0].Affected || simResp.Items[0].NetDelta == nil || !simResp.Items[0].NetDelta.IsPositive() {
		t.Fatalf("expected positive net delta for 22->20 days: %+v", simResp.Items[0])
	}
	if simResp.Items[1].Status != "uncomputable" || simResp.Items[1].NetDelta != nil {
		t.Fatalf("expected rel2 uncomputable without zero coercion: %+v", simResp.Items[1])
	}
	var payslipCountAfter, runCountAfter int
	_ = pool.QueryRow(ctx, `SELECT count(*) FROM payslips WHERE employment_id = ANY($1::uuid[])`, []uuid.UUID{employment1ID, employment2ID}).Scan(&payslipCountAfter)
	_ = pool.QueryRow(ctx, `SELECT count(*) FROM payroll_calculation_runs WHERE employment_id = ANY($1::uuid[])`, []uuid.UUID{employment1ID, employment2ID}).Scan(&runCountAfter)
	if payslipCountAfter != payslipCountBefore || runCountAfter != runCountBefore {
		t.Fatalf("simulation wrote DB rows: payslips %d->%d runs %d->%d", payslipCountBefore, payslipCountAfter, runCountBefore, runCountAfter)
	}

	// 5. Atomic membership transfer for open period (2026-05-01) -> closes old membership & opens new
	draftB1, err := wpRepo.CreateWorkPolicyGroup(ctx, workpolicy.CreateGroupInput{
		Code: "WPV2-B-" + uuid.NewString()[:6], Name: "Group B", Draft: draftA2Fields,
	}, ids.UserID(actorID))
	if err != nil {
		t.Fatalf("create group B: %v", err)
	}
	if _, err := wpRepo.PublishWorkPolicyVersion(ctx, workpolicy.PublishVersionInput{
		GroupID: workpolicy.GroupID(draftB1.GroupID), VersionID: workpolicy.VersionID(draftB1.ID), ExpectedRowVersion: draftB1.RowVersion, ActorID: ids.UserID(actorID),
	}); err != nil {
		t.Fatalf("publish B1: %v", err)
	}
	// Transfer touching locked 2026-03 rejected
	if _, err := wpRepo.TransferWorkPolicyMembership(ctx, workpolicy.TransferMembershipInput{
		EmploymentRelationshipID: rel1ID, TargetGroupID: workpolicy.GroupID(draftB1.GroupID), EffectiveDate: "2026-03-15", ActorID: ids.UserID(actorID),
	}); !errors.Is(err, api.ErrWorkPolicyLockedPeriod) {
		t.Fatalf("expected locked period conflict on transfer, got %v", err)
	}
	// Transfer on 2026-05-01 succeeds atomically
	transferRes, err := wpRepo.TransferWorkPolicyMembership(ctx, workpolicy.TransferMembershipInput{
		EmploymentRelationshipID: rel1ID, TargetGroupID: workpolicy.GroupID(draftB1.GroupID), EffectiveDate: "2026-05-01", ActorID: ids.UserID(actorID),
	})
	if err != nil || transferRes.ClosedMembership == nil || *transferRes.ClosedMembership.EffectiveTo != "2026-05-01" || transferRes.NewMembership.GroupID != draftB1.GroupID {
		t.Fatalf("unexpected transfer result: %+v err=%v", transferRes, err)
	}
	resolvedMay, err := (WorkPolicyResolver{}).ResolveAt(ctx, pool, rel1ID, time.Date(2026, 5, 1, 0, 0, 0, 0, time.UTC))
	if err != nil || resolvedMay.GroupID != draftB1.GroupID {
		t.Fatalf("expected Group B on 2026-05-01, got %+v err=%v", resolvedMay, err)
	}
}
