//go:build integration

package repository

import (
	"context"
	"errors"
	"fmt"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type recordingSessionRevoker struct {
	revokedUsers []uuid.UUID
	err          error
}

func (r *recordingSessionRevoker) SetUserRevocation(_ context.Context, userID uuid.UUID, _ time.Duration) error {
	if r.err != nil {
		return r.err
	}
	r.revokedUsers = append(r.revokedUsers, userID)
	return nil
}

func TestIntegrationTenuresV2_PermanentCodeOwnershipAndLifecycle(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin tx: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	scope := seedAtomicWriteScope(t, ctx, tx)
	revoker := &recordingSessionRevoker{}
	empRepo := EmployeeRepository{
		DB:      tx,
		BeginTx: func(context.Context) (service.TxBeginner, error) { return tx, nil },
		Revoker: revoker,
	}
	employmentRepo := EmploymentRepository{
		DB:      tx,
		BeginTx: func(context.Context) (TxBeginner, error) { return tx, nil },
		Revoker: revoker,
	}

	userA := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash, full_name) VALUES ($1, $2, 'hash', 'Tenure User A')`, userA, fmt.Sprintf("tenure-a-%s@example.invalid", userA.String()[:8])); err != nil {
		t.Fatalf("insert userA: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO sessions (id, user_id, token, expires_at) VALUES ($1, $2, $3, NOW() + INTERVAL '1 day')`, uuid.New(), userA, "tok-"+userA.String()); err != nil {
		t.Fatalf("insert session for userA: %v", err)
	}

	// 1. Initial hire under Employer A with linked userA.
	personA, err := empRepo.CreateEmployee(ctx, api.CreateEmployeeRequest{
		FullName:          "Person A",
		EmployeeIDNumber:  "EMP-V2-001",
		HireDate:          "2026-01-01",
		UserID:            &userA,
		EmployerID:        &scope.employerA,
		DepartmentID:      &scope.department,
		PrimaryLocationID: &scope.location,
	})
	if err != nil {
		t.Fatalf("CreateEmployee personA: %v", err)
	}
	if personA.ActiveEmploymentID == nil || personA.TenureID == nil {
		t.Fatalf("expected active employment & tenure IDs: %+v", personA)
	}

	// 2. Add second concurrent employer (Employer B) under the SAME tenure.
	relB, err := employmentRepo.CreateEmploymentRelationship(ctx, personA.ID, api.CreateEmploymentRelationshipRequest{
		EmployerID:     scope.employerB,
		Status:         "active",
		StartDate:      "2026-02-01",
		EmploymentType: "full_time",
	})
	if err != nil {
		t.Fatalf("CreateEmploymentRelationship employerB: %v", err)
	}
	if relB.Relationship.TenureID != *personA.TenureID {
		t.Fatalf("employerB tenure_id = %s, want %s", relB.Relationship.TenureID, *personA.TenureID)
	}

	// 3. Terminate Employer A only -> relationship closes, tenure MUST stay open, session NOT revoked.
	termA, err := employmentRepo.TerminateEmployment(ctx, personA.ID, *personA.ActiveEmploymentID, api.TerminateEmploymentRequest{
		TerminationDate:   "2026-03-31",
		TerminationReason: strPtr("end_of_assignment"),
	})
	if err != nil {
		t.Fatalf("TerminateEmployment employerA: %v", err)
	}
	if !termA.RelationshipClosed || termA.TenureClosed || termA.SessionsRevoked {
		t.Fatalf("multi-employer first terminate = %+v, want relClosed=true tenureClosed=false sessionsRevoked=false", termA)
	}

	// 4. Terminate Employer B (last open relationship) -> tenure auto-closes AND userA sessions are revoked.
	termB, err := employmentRepo.TerminateEmployment(ctx, personA.ID, relB.Segment.ID, api.TerminateEmploymentRequest{
		TerminationDate:   "2026-04-30",
		TerminationReason: strPtr("voluntary_resignation"),
	})
	if err != nil {
		t.Fatalf("TerminateEmployment employerB: %v", err)
	}
	if !termB.RelationshipClosed || !termB.TenureClosed || !termB.SessionsRevoked {
		t.Fatalf("last employer terminate = %+v, want all true", termB)
	}
	if len(revoker.revokedUsers) != 1 || revoker.revokedUsers[0] != userA {
		t.Fatalf("revoked users = %v, want [%s]", revoker.revokedUsers, userA)
	}
	var openSessions int
	if err := tx.QueryRow(ctx, `SELECT count(*) FROM sessions WHERE user_id = $1 AND revoked_at IS NULL`, userA).Scan(&openSessions); err != nil || openSessions != 0 {
		t.Fatalf("open db sessions after tenure close = %d (err=%v), want 0", openSessions, err)
	}

	// 5. Permanent cross-person code ban: Person B CANNOT use " emp-v2-001 " even after Person A terminated!
	if _, err := empRepo.CreateEmployee(ctx, api.CreateEmployeeRequest{
		FullName:         "Person B",
		EmployeeIDNumber: " emp-v2-001 ",
		HireDate:         "2026-06-01",
		EmployerID:       &scope.employerA,
	}); !errors.Is(err, api.ErrConflict) {
		t.Fatalf("cross-person code reuse after termination err = %v, want ErrConflict", err)
	}

	// 6. Rehire Person A with a NEW employee number + prior_service_credit_days.
	credit := 120
	rehire, err := empRepo.CreateEmployeeTenure(ctx, personA.ID, api.CreateEmployeeTenureRequest{
		StartDate:              "2026-06-01",
		EmployerID:             scope.employerA,
		EmployeeNumber:         strPtr("EMP-V2-001-R2"),
		PriorServiceCreditDays: &credit,
		Notes:                  strPtr("Rehired senior"),
	})
	if err != nil {
		t.Fatalf("CreateEmployeeTenure rehire with new code: %v", err)
	}
	if rehire.HireType != "rehire" || rehire.EmployeeCode != "EMP-V2-001-R2" || rehire.PriorServiceCreditDays != 120 {
		t.Fatalf("unexpected rehire result: %+v", rehire)
	}

	// 7. ListEmployeeTenures returns both tenures newest-first with relationships intact.
	tenures, err := empRepo.ListEmployeeTenures(ctx, personA.ID)
	if err != nil {
		t.Fatalf("ListEmployeeTenures: %v", err)
	}
	if len(tenures) != 2 || tenures[0].ID != rehire.TenureID || tenures[0].HireType != "rehire" || tenures[1].ID != *personA.TenureID || len(tenures[1].Relationships) != 2 {
		t.Fatalf("unexpected tenures list: %+v", tenures)
	}

	// 8. Explicit CloseTenure with rehire_eligible=false blocks future rehire.
	notEligible := false
	closed, err := empRepo.CloseEmployeeTenure(ctx, personA.ID, rehire.TenureID, api.CloseEmployeeTenureRequest{
		EndDate:           "2026-08-31",
		TerminationReason: strPtr("misconduct"),
		RehireEligible:    &notEligible,
	})
	if err != nil {
		t.Fatalf("CloseEmployeeTenure: %v", err)
	}
	if closed.ClosedRelationships != 1 || closed.ClosedSegments != 1 {
		t.Fatalf("unexpected CloseEmployeeTenure counts: %+v", closed)
	}
	if _, err := empRepo.CreateEmployeeTenure(ctx, personA.ID, api.CreateEmployeeTenureRequest{
		StartDate:  "2026-09-15",
		EmployerID: scope.employerA,
	}); !errors.Is(err, api.ErrConflict) {
		t.Fatalf("rehire when rehire_eligible=false err = %v, want ErrConflict", err)
	}
}

func TestIntegrationTenuresV2_ESSDocumentIsolationAndUnlinkAuditTrail(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin tx: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	scope := seedAtomicWriteScope(t, ctx, tx)
	revoker := &recordingSessionRevoker{}
	rbacRepo := OrganizationRBACRepository{SystemDB: tx, Revoker: revoker}
	essRepo := ESSRepository{DB: tx}

	empID := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO employees (id, full_name) VALUES ($1, 'ESS Isolated Person')`, empID); err != nil {
		t.Fatalf("insert employee: %v", err)
	}

	userOld, userNew := uuid.New(), uuid.New()
	var employeeRoleID uuid.UUID
	if err := tx.QueryRow(ctx, `SELECT id FROM roles WHERE name = 'employee' LIMIT 1`).Scan(&employeeRoleID); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			employeeRoleID = uuid.New()
			if _, err := tx.Exec(ctx, `INSERT INTO roles (id, name, description, color, is_system, scope_type) VALUES ($1, 'employee', 'Employee', 'gray', true, 'scoped')`, employeeRoleID); err != nil {
				t.Fatalf("insert employee role: %v", err)
			}
		} else {
			t.Fatalf("query employee role: %v", err)
		}
	}
	for _, u := range []uuid.UUID{userOld, userNew} {
		if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash, full_name) VALUES ($1, $2, 'hash', 'ESS User')`, u, fmt.Sprintf("ess-%s@example.invalid", u.String()[:8])); err != nil {
			t.Fatalf("insert user: %v", err)
		}
		if _, err := tx.Exec(ctx, `INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)`, u, employeeRoleID); err != nil {
			t.Fatalf("insert user_role: %v", err)
		}
	}

	// Tenure 1 (closed, linked to userOld) + Tenure 2 (active, linked to userNew via RBAC repo).
	tenure1, tenure2 := uuid.New(), uuid.New()
	if _, err := tx.Exec(ctx, `
		INSERT INTO employee_tenures (id, employee_id, employee_number, user_id, hire_type, start_date, end_date)
		VALUES
			($1, $3, 'ESS-ISO-1', $4, 'initial', DATE '2025-01-01', DATE '2025-12-31'),
			($2, $3, 'ESS-ISO-1', NULL, 'rehire', DATE '2026-01-15', NULL)
	`, tenure1, tenure2, empID, userOld); err != nil {
		t.Fatalf("insert tenures: %v", err)
	}
	if err := rbacRepo.LinkMemberEmployee(ctx, userNew, userNew, empID); err != nil {
		t.Fatalf("LinkMemberEmployee: %v", err)
	}

	// Seed documents and pending acknowledgements in both Tenure 1 and Tenure 2.
	docOld, docNew := uuid.New(), uuid.New()
	if _, err := tx.Exec(ctx, `
		INSERT INTO employee_documents (id, employee_id, tenure_id, document_type, file_name, file_path, file_size, mime_type, is_active)
		VALUES
			($1, $3, $4, 'contract', 'old-contract.pdf', '/tmp/old.pdf', 100, 'application/pdf', true),
			($2, $3, $5, 'contract', 'new-contract.pdf', '/tmp/new.pdf', 100, 'application/pdf', true)
	`, docOld, docNew, empID, tenure1, tenure2); err != nil {
		t.Fatalf("insert employee_documents: %v", err)
	}

	fileOld, fileNew := uuid.New(), uuid.New()
	for _, f := range []uuid.UUID{fileOld, fileNew} {
		if _, err := tx.Exec(ctx, `INSERT INTO files (id, file_name, storage_key, file_size, mime_type, category, owner_type, owner_id) VALUES ($1, 'f.pdf', $2, 10, 'application/pdf', 'policy', 'employee', $3)`, f, "files/"+f.String()+".pdf", empID); err != nil {
			t.Fatalf("insert file: %v", err)
		}
	}
	ackOld, ackNew := uuid.New(), uuid.New()
	if _, err := tx.Exec(ctx, `
		INSERT INTO file_acknowledgements (id, document_id, employee_id, tenure_id, status)
		VALUES
			($1, $3, $5, $6, 'pending'),
			($2, $4, $5, $7, 'pending')
	`, ackOld, ackNew, fileOld, fileNew, empID, tenure1, tenure2); err != nil {
		t.Fatalf("insert file_acknowledgements: %v", err)
	}

	// Old user from closed tenure cannot resolve ESS employee.
	if _, err := essRepo.ResolveEmployeeForUser(ctx, userOld); !errors.Is(err, api.ErrESSNoEmployeeRecord) {
		t.Fatalf("old tenure user ResolveEmployeeForUser err = %v, want ErrESSNoEmployeeRecord", err)
	}

	// New user sees ONLY Tenure 2 documents and acknowledgements, and CANNOT acknowledge Tenure 1 ack by direct ID.
	docs, err := essRepo.GetEmployeeDocuments(ctx, empID)
	if err != nil || len(docs) != 1 || docs[0].ID != docNew {
		t.Fatalf("ESS GetEmployeeDocuments = (%+v, %v), want only docNew %s", docs, err, docNew)
	}
	acks, err := essRepo.GetPendingDocuments(ctx, empID)
	if err != nil || len(acks) != 1 || acks[0].ID != ackNew {
		t.Fatalf("ESS GetPendingDocuments = (%+v, %v), want only ackNew %s", acks, err, ackNew)
	}
	if err := essRepo.AcknowledgeDocument(ctx, empID, ackOld); !errors.Is(err, api.ErrESSNotFound) {
		t.Fatalf("AcknowledgeDocument cross-tenure err = %v, want ErrESSNotFound", err)
	}

	// Unlinking userNew clears ONLY Tenure 2, preserves Tenure 1 userOld audit trail, and revokes userNew session.
	if _, err := tx.Exec(ctx, `INSERT INTO sessions (id, user_id, token, expires_at) VALUES ($1, $2, $3, NOW() + INTERVAL '1 day')`, uuid.New(), userNew, "tok-"+userNew.String()); err != nil {
		t.Fatalf("insert session for userNew: %v", err)
	}
	if err := rbacRepo.UnlinkMemberEmployee(ctx, userNew, userNew); err != nil {
		t.Fatalf("UnlinkMemberEmployee: %v", err)
	}
	var t1User, t2User *uuid.UUID
	if err := tx.QueryRow(ctx, `SELECT user_id FROM employee_tenures WHERE id = $1`, tenure1).Scan(&t1User); err != nil {
		t.Fatalf("read tenure1 user: %v", err)
	}
	if err := tx.QueryRow(ctx, `SELECT user_id FROM employee_tenures WHERE id = $1`, tenure2).Scan(&t2User); err != nil {
		t.Fatalf("read tenure2 user: %v", err)
	}
	if t1User == nil || *t1User != userOld {
		t.Fatalf("historical tenure1 user_id was wiped: got %v, want %s", t1User, userOld)
	}
	if t2User != nil {
		t.Fatalf("open tenure2 user_id not cleared: got %v", t2User)
	}
	if len(revoker.revokedUsers) != 1 || revoker.revokedUsers[0] != userNew {
		t.Fatalf("revoked users after unlink = %v, want [%s]", revoker.revokedUsers, userNew)
	}
	_ = scope
}
