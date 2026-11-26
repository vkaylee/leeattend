//go:build integration

package repository

import (
	"context"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgconn"
)

type integrationExecer interface {
	Exec(ctx context.Context, sql string, arguments ...any) (pgconn.CommandTag, error)
}

// seedIntegrationEmployeeGraph inserts a valid 4-layer HRM fixture
// (employees -> employee_tenures -> employment_relationships -> employments)
// so integration tests satisfy all Employee Tenures v2 NOT NULL and containment invariants.
func seedIntegrationEmployeeGraph(
	t *testing.T,
	ctx context.Context,
	db DBTX,
	employeeID, employerID, employmentID uuid.UUID,
	employeeNumber, fullName, startDate string,
	departmentID, locationID *uuid.UUID,
) (tenureID, relationshipID uuid.UUID) {
	t.Helper()
	tenureID = uuid.New()
	relationshipID = uuid.New()

	if _, err := db.Exec(ctx, `INSERT INTO employees (id, full_name) VALUES ($1, $2)`, employeeID, fullName); err != nil {
		t.Fatalf("seed employees: %v", err)
	}
	if _, err := db.Exec(ctx, `
		INSERT INTO employee_tenures (id, employee_id, employee_number, hire_type, start_date)
		VALUES ($1, $2, $3, 'initial', $4::date)
	`, tenureID, employeeID, employeeNumber, startDate); err != nil {
		t.Fatalf("seed employee_tenures: %v", err)
	}
	if _, err := db.Exec(ctx, `
		INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date)
		VALUES ($1, $2, $3, $4, 'active', $5::date)
	`, relationshipID, employeeID, employerID, tenureID, startDate); err != nil {
		t.Fatalf("seed employment_relationships: %v", err)
	}
	if _, err := db.Exec(ctx, `
		INSERT INTO employment_segments (id, employee_id, employer_id, relationship_id, department_id, primary_location_id, status, start_date, employment_type)
		VALUES ($1, $2, $3, $4, $5, $6, 'active', $7::date, 'full_time')
	`, employmentID, employeeID, employerID, relationshipID, departmentID, locationID, startDate); err != nil {
		t.Fatalf("seed employments: %v", err)
	}
	return tenureID, relationshipID
}

// seedIntegrationEmployeeGraphWithRange inserts a valid 4-layer HRM fixture where the
// parent tenure and relationship start on tenureStartDate (open-ended) and contain
// the child employment segment [employmentStartDate, employmentEndDate] ("" = open-ended).
func seedIntegrationEmployeeGraphWithRange(
	t *testing.T,
	ctx context.Context,
	db integrationExecer,
	employeeID, employerID, employmentID uuid.UUID,
	employeeNumber, fullName, tenureStartDate, employmentStartDate, employmentEndDate string,
) (tenureID, relationshipID uuid.UUID) {
	t.Helper()
	tenureID = uuid.New()
	relationshipID = uuid.New()

	if _, err := db.Exec(ctx, `INSERT INTO employees (id, full_name) VALUES ($1, $2)`, employeeID, fullName); err != nil {
		t.Fatalf("seed employees: %v", err)
	}
	if _, err := db.Exec(ctx, `
		INSERT INTO employee_tenures (id, employee_id, employee_number, hire_type, start_date)
		VALUES ($1, $2, $3, 'initial', $4::date)
	`, tenureID, employeeID, employeeNumber, tenureStartDate); err != nil {
		t.Fatalf("seed employee_tenures: %v", err)
	}
	if _, err := db.Exec(ctx, `
		INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date)
		VALUES ($1, $2, $3, $4, 'active', $5::date)
	`, relationshipID, employeeID, employerID, tenureID, tenureStartDate); err != nil {
		t.Fatalf("seed employment_relationships: %v", err)
	}
	if _, err := db.Exec(ctx, `
		INSERT INTO employment_segments (id, employee_id, employer_id, relationship_id, status, start_date, end_date, employment_type)
		VALUES ($1, $2, $3, $4, 'active', $5::date, NULLIF($6, '')::date, 'full_time')
	`, employmentID, employeeID, employerID, relationshipID, employmentStartDate, employmentEndDate); err != nil {
		t.Fatalf("seed employments: %v", err)
	}
	return tenureID, relationshipID
}
