package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	goredis "github.com/redis/go-redis/v9"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	redisinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/redis"
)

type EmployeeImportRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
	BeginTx  func(context.Context, uuid.UUID) (TxBeginner, error)
}

func (r EmployeeImportRepository) db(ctx context.Context, wsID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("employee import repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, wsID)
}

func (r EmployeeImportRepository) begin(ctx context.Context, wsID uuid.UUID) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx, wsID)
	}
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("employee import repository: database does not support transactions")
	}
	return beginner, nil
}

func (r EmployeeImportRepository) CreateEmployeeImportJob(ctx context.Context, workspaceID, userID uuid.UUID, filePath string, jobID uuid.UUID) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO attendance_import_jobs
		(id, workspace_id, user_id, status, total_rows, processed_rows, file_path, target_type, created_at, updated_at)
		VALUES ($1, $2, $3, 'pending', 0, 0, $4, 'employee', NOW(), NOW())`,
		jobID, workspaceID, userID, filePath)
	return err
}

func (r EmployeeImportRepository) TerminationCandidates(ctx context.Context, workspaceID uuid.UUID, codes []string) ([]api.TerminationCandidate, error) {
	if len(codes) == 0 {
		return nil, nil
	}
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	upperCodes := make([]string, len(codes))
	for i, c := range codes {
		upperCodes[i] = strings.ToUpper(strings.TrimSpace(c))
	}

	empRows, err := db.Query(ctx, `SELECT id, employee_id_number, full_name
		FROM employees
		WHERE workspace_id = $1 AND deleted_at IS NULL AND UPPER(employee_id_number) = ANY($2)`,
		workspaceID, upperCodes)
	if err != nil {
		return nil, err
	}
	defer empRows.Close()

	candidatesByEmpID := make(map[uuid.UUID]*api.TerminationCandidate)
	candidateList := make([]*api.TerminationCandidate, 0)
	empIDs := make([]uuid.UUID, 0)

	for empRows.Next() {
		var (
			id   uuid.UUID
			code string
			name string
		)
		if err := empRows.Scan(&id, &code, &name); err != nil {
			return nil, err
		}
		candidate := &api.TerminationCandidate{
			EmployeeID:   id,
			EmployeeCode: code,
			FullName:     name,
			Employments:  make([]api.TerminationEmployment, 0),
			Balances:     make(map[int][]api.TerminationLeaveBalance),
		}
		candidatesByEmpID[id] = candidate
		candidateList = append(candidateList, candidate)
		empIDs = append(empIDs, id)
	}
	if err := empRows.Err(); err != nil {
		return nil, err
	}
	if len(empIDs) == 0 {
		return nil, nil
	}

	employmentsRows, err := db.Query(ctx, `SELECT id, employee_id, status, start_date
		FROM employments
		WHERE workspace_id = $1 AND employee_id = ANY($2) AND deleted_at IS NULL
		ORDER BY start_date DESC`,
		workspaceID, empIDs)
	if err != nil {
		return nil, err
	}
	defer employmentsRows.Close()

	employmentToCandidate := make(map[uuid.UUID]*api.TerminationCandidate)
	employmentIDs := make([]uuid.UUID, 0)

	for employmentsRows.Next() {
		var (
			empID      uuid.UUID
			employeeID uuid.UUID
			status     string
			startDate  time.Time
		)
		if err := employmentsRows.Scan(&empID, &employeeID, &status, &startDate); err != nil {
			return nil, err
		}
		if cand, ok := candidatesByEmpID[employeeID]; ok {
			cand.Employments = append(cand.Employments, api.TerminationEmployment{
				ID:        empID,
				Status:    status,
				StartDate: startDate,
			})
			employmentToCandidate[empID] = cand
			employmentIDs = append(employmentIDs, empID)
		}
	}
	if err := employmentsRows.Err(); err != nil {
		return nil, err
	}

	if len(employmentIDs) > 0 {
		balanceRows, err := db.Query(ctx, `SELECT lb.employment_id, lb.year, lb.entitlement, lt.id, lt.code, lt.name
			FROM leave_balances lb
			JOIN leave_types lt ON lt.id = lb.leave_type_id
			WHERE lb.employment_id = ANY($1)`,
			employmentIDs)
		if err != nil {
			return nil, err
		}
		defer balanceRows.Close()

		for balanceRows.Next() {
			var (
				employmentID uuid.UUID
				year         int
				entitlement  int
				typeID       uuid.UUID
				typeCode     string
				typeName     string
			)
			if err := balanceRows.Scan(&employmentID, &year, &entitlement, &typeID, &typeCode, &typeName); err != nil {
				return nil, err
			}
			if cand, ok := employmentToCandidate[employmentID]; ok {
				cand.Balances[year] = append(cand.Balances[year], api.TerminationLeaveBalance{
					LeaveTypeID:   typeID,
					LeaveTypeCode: typeCode,
					LeaveTypeName: typeName,
					Entitlement:   entitlement,
				})
			}
		}
		if err := balanceRows.Err(); err != nil {
			return nil, err
		}
	}

	result := make([]api.TerminationCandidate, len(candidateList))
	for i, c := range candidateList {
		result[i] = *c
	}
	return result, nil
}

func (r EmployeeImportRepository) TerminationStatuses(ctx context.Context, workspaceID uuid.UUID, employmentIDs []uuid.UUID) (map[uuid.UUID]string, error) {
	if len(employmentIDs) == 0 {
		return make(map[uuid.UUID]string), nil
	}
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, status FROM employments WHERE workspace_id = $1 AND id = ANY($2) AND deleted_at IS NULL`,
		workspaceID, employmentIDs)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make(map[uuid.UUID]string, len(employmentIDs))
	for rows.Next() {
		var id uuid.UUID
		var status string
		if err := rows.Scan(&id, &status); err != nil {
			return nil, err
		}
		result[id] = status
	}
	return result, rows.Err()
}

func (r EmployeeImportRepository) TerminateEmploymentsTransactional(ctx context.Context, workspaceID uuid.UUID, rows []api.TerminationJobRow) (int, error) {
	if len(rows) == 0 {
		return 0, nil
	}
	txBeginner, err := r.begin(ctx, workspaceID)
	if err != nil {
		return 0, err
	}
	tx, err := txBeginner.Begin(ctx)
	if err != nil {
		return 0, err
	}
	defer func() { _ = tx.Rollback(ctx) }()

	successCount := 0
	for _, row := range rows {
		termDate, err := time.Parse("2006-01-02", row.TerminationDate)
		if err != nil {
			return successCount, fmt.Errorf("invalid termination date %s: %w", row.TerminationDate, err)
		}
		var startDate time.Time
		var currentStatus string
		err = tx.QueryRow(ctx, `SELECT status, start_date FROM employments WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL FOR UPDATE`,
			row.EmploymentID, workspaceID).Scan(&currentStatus, &startDate)
		if err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				continue
			}
			return successCount, err
		}
		if currentStatus == "terminated" {
			continue
		}

		_, err = tx.Exec(ctx, `UPDATE employments
			SET status = 'terminated', end_date = $1::date, termination_reason = $2, notes = $3, updated_at = NOW()
			WHERE id = $4 AND workspace_id = $5 AND deleted_at IS NULL`,
			row.TerminationDate, row.TerminationReason, row.TerminationNotes, row.EmploymentID, workspaceID)
		if err != nil {
			return successCount, err
		}

		year := termDate.Year()
		startOfYear := time.Date(year, 1, 1, 0, 0, 0, 0, time.UTC)
		endOfYear := time.Date(year, 12, 31, 0, 0, 0, 0, time.UTC)
		effectiveStart := startDate
		if effectiveStart.Before(startOfYear) {
			effectiveStart = startOfYear
		}
		daysWorked := int(termDate.Sub(effectiveStart).Hours()/24) + 1
		totalDays := int(endOfYear.Sub(startOfYear).Hours()/24) + 1

		balRows, err := tx.Query(ctx, `SELECT id, leave_type_id, entitlement FROM leave_balances WHERE employment_id = $1 AND year = $2`,
			row.EmploymentID, year)
		if err != nil {
			return successCount, err
		}
		type existingBal struct {
			id          uuid.UUID
			leaveTypeID uuid.UUID
			entitlement int
		}
		bals := make([]existingBal, 0)
		for balRows.Next() {
			var b existingBal
			if err := balRows.Scan(&b.id, &b.leaveTypeID, &b.entitlement); err != nil {
				balRows.Close()
				return successCount, err
			}
			bals = append(bals, b)
		}
		balRows.Close()

		for _, b := range bals {
			prorated := 0
			if daysWorked > 0 && totalDays > 0 {
				prorated = (b.entitlement*daysWorked + totalDays/2) / totalDays
			}
			_, err = tx.Exec(ctx, `UPDATE leave_balances
				SET entitlement = $1, updated_at = NOW()
				WHERE id = $2`,
				prorated, b.id)
			if err != nil {
				return successCount, err
			}
		}

		successCount++
	}

	if err := tx.Commit(ctx); err != nil {
		return 0, err
	}
	return successCount, nil
}

// EmployeeImportQueue publishes Rust-compatible worker envelopes onto the shared
// Redis job queues consumed by the apiserver worker.
type EmployeeImportQueue struct {
	RedisClient *redisinfra.Client
	Repo        EmployeeImportRepository
}

// queuedJob mirrors the JSON shape of the Rust `Job` envelope. Field names and
// ordering of required fields are part of the cross-language queue contract.
type queuedJob[T any] struct {
	JobID        uuid.UUID           `json:"job_id"`
	ParentJobID  uuid.UUID           `json:"parent_job_id"`
	BatchIndex   int                 `json:"batch_index"`
	TaskType     string              `json:"task_type"`
	WorkspaceID  uuid.UUID           `json:"workspace_id"`
	Payload      queuedJobPayload[T] `json:"payload"`
	Priority     string              `json:"priority"`
	CreatedAt    string              `json:"created_at"`
	RetryCount   int                 `json:"retry_count"`
	MaxRetries   int                 `json:"max_retries"`
	TraceContext *string             `json:"trace_context"`
}

// queuedJobPayload mirrors the Rust `JobPayload` tagged enum.
type queuedJobPayload[T any] struct {
	Type string `json:"type"`
	Data T      `json:"data"`
}

type employeeImportJobData struct {
	ImportJobID uuid.UUID `json:"import_job_id"`
	UserID      uuid.UUID `json:"user_id"`
}

type terminationJobData struct {
	Rows []api.TerminationJobRow `json:"rows"`
}

// jobMetadata mirrors the Rust `JobMetadata` Redis record.
type jobMetadata struct {
	JobID            uuid.UUID `json:"job_id"`
	Status           string    `json:"status"`
	TotalBatches     int       `json:"total_batches"`
	CompletedBatches int       `json:"completed_batches"`
	FailedBatches    int       `json:"failed_batches"`
	WorkspaceID      uuid.UUID `json:"workspace_id"`
	RequesterID      uuid.UUID `json:"requester_id"`
	DateRange        *string   `json:"date_range"`
	CreatedAt        string    `json:"created_at"`
	CompletedAt      *string   `json:"completed_at"`
	ErrorMessage     *string   `json:"error_message"`
	TaskType         string    `json:"task_type"`
	Payload          *string   `json:"payload,omitempty"`
}

func (q EmployeeImportQueue) shared(ctx context.Context) (*goredis.Client, error) {
	if q.RedisClient == nil {
		return nil, errors.New("employee import queue: redis is not configured")
	}
	shared, err := q.RedisClient.Shared()
	if err != nil {
		return nil, err
	}
	if shared == nil {
		return nil, errors.New("employee import queue: redis client is not available")
	}
	return shared, nil
}

func (q EmployeeImportQueue) publish(ctx context.Context, envelope any) error {
	shared, err := q.shared(ctx)
	if err != nil {
		return err
	}
	data, err := json.Marshal(envelope)
	if err != nil {
		return err
	}
	return shared.RPush(ctx, redisinfra.KeyWorkerQueue("", redisinfra.PriorityNormal), data).Err()
}

func (q EmployeeImportQueue) EnqueueEmployeeImport(ctx context.Context, job api.EmployeeImportJob) error {
	envelope := queuedJob[employeeImportJobData]{
		JobID:        job.JobID,
		ParentJobID:  job.JobID,
		TaskType:     "employee_import",
		WorkspaceID:  job.WorkspaceID,
		Payload:      queuedJobPayload[employeeImportJobData]{Type: "EmployeeImport", Data: employeeImportJobData{ImportJobID: job.JobID, UserID: job.UserID}},
		Priority:     "normal",
		CreatedAt:    time.Now().UTC().Format(time.RFC3339),
		MaxRetries:   3,
		TraceContext: nil,
	}
	return q.publish(ctx, envelope)
}

func (q EmployeeImportQueue) EnqueueTerminationImport(ctx context.Context, job api.TerminationImportJob) error {
	shared, err := q.shared(ctx)
	if err != nil {
		// Durable fallback: apply the same transactional termination effects the
		// worker would perform when the queue backend is unavailable.
		_, syncErr := q.Repo.TerminateEmploymentsTransactional(ctx, job.WorkspaceID, job.Rows)
		return syncErr
	}
	meta := jobMetadata{
		JobID:        job.JobID,
		Status:       "pending",
		TotalBatches: len(job.Rows),
		WorkspaceID:  job.WorkspaceID,
		RequesterID:  job.UserID,
		CreatedAt:    time.Now().UTC().Format(time.RFC3339),
		TaskType:     "bulk_termination",
	}
	metaJSON, err := json.Marshal(meta)
	if err != nil {
		return err
	}
	if err := shared.Set(ctx, redisinfra.KeyReportMeta(job.JobID), metaJSON, 24*time.Hour).Err(); err != nil {
		return err
	}
	envelope := queuedJob[terminationJobData]{
		JobID:        job.JobID,
		ParentJobID:  job.JobID,
		TaskType:     "bulk_termination",
		WorkspaceID:  job.WorkspaceID,
		Payload:      queuedJobPayload[terminationJobData]{Type: "BulkTermination", Data: terminationJobData{Rows: job.Rows}},
		Priority:     "normal",
		CreatedAt:    time.Now().UTC().Format(time.RFC3339),
		MaxRetries:   3,
		TraceContext: nil,
	}
	data, err := json.Marshal(envelope)
	if err != nil {
		return err
	}
	return shared.RPush(ctx, redisinfra.KeyWorkerQueue("", redisinfra.PriorityNormal), data).Err()
}
