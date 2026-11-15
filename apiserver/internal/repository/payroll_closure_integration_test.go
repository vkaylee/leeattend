//go:build integration

package repository

import (
	"context"
	"errors"
	"testing"

	"github.com/google/uuid"
	api "github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	payrolldomain "github.com/vkaylee/leeattend/apiserver/internal/domain/payroll"
)

func TestIntegrationPayrollLockRequiresClosedAttendancePeriod(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, actorID := uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'payroll-closure-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert actor: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO payroll_periods (id, employer_id, period_year, period_month, status, version) VALUES ($1, $2, 2026, 11, 'open', 1)`, uuid.New(), employerID); err != nil {
		t.Fatalf("insert payroll period: %v", err)
	}
	repo := PayrollRepository{DB: pool}
	_, err := repo.LockPayrollPeriod(ctx, payrolldomain.PayrollPeriodMutation{EmployerID: ids.EmployerID(employerID), Year: 2026, Month: 11, ExpectedVersion: 1, ActorID: ids.UserID(actorID)})
	if !errors.Is(err, api.ErrPayrollPeriodConflict) {
		t.Fatalf("open attendance lock error = %v, want conflict", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO attendance_periods (id, employer_id, period_year, period_month, status, version) VALUES ($1, $2, 2026, 11, 'closed', 2)`, uuid.New(), employerID); err != nil {
		t.Fatalf("insert closed attendance: %v", err)
	}
	period, err := repo.LockPayrollPeriod(ctx, payrolldomain.PayrollPeriodMutation{EmployerID: ids.EmployerID(employerID), Year: 2026, Month: 11, ExpectedVersion: 1, ActorID: ids.UserID(actorID)})
	if err != nil {
		t.Fatalf("closed attendance payroll lock: %v", err)
	}
	if period.Status != "locked" || period.Version != 2 {
		t.Fatalf("payroll period after lock: %+v", period)
	}
}

var _ context.Context
