package repository

import (
	"context"
	"crypto/sha256"
	"encoding/json"
	"errors"
	"fmt"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
	"sort"
	"strings"
	"time"
)

func findIdempotentPayrollInputBatch(ctx context.Context, tx pgx.Tx, idempotencyKey, requestHash string) (api.PayrollInputBatch, bool, error) {
	var existingHash string
	var existingID uuid.UUID
	err := tx.QueryRow(ctx, `SELECT id, request_hash FROM payroll_input_batches WHERE idempotency_key = $1 FOR UPDATE`, idempotencyKey).Scan(&existingID, &existingHash)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.PayrollInputBatch{}, false, nil
	}
	if err != nil {
		return api.PayrollInputBatch{}, false, err
	}
	if existingHash != requestHash {
		return api.PayrollInputBatch{}, false, api.ErrPayrollInputConflict
	}
	batch, scanErr := scanPayrollInputBatch(tx.QueryRow(ctx, `SELECT b.id, b.period_year, b.period_month, b.source, b.idempotency_key, b.status, count(i.id), b.submitted_by, b.approved_by, b.locked_by, b.created_at
		FROM payroll_input_batches b LEFT JOIN payroll_inputs i ON i.batch_id = b.id
		WHERE b.id = $1 GROUP BY b.id`, existingID))
	return batch, scanErr == nil, scanErr
}

func validatePayrollInputMetrics(ctx context.Context, tx pgx.Tx, inputs []api.PayrollInput, metricCodes []string) error {
	rows, err := tx.Query(ctx, `SELECT code, unit FROM payroll_metric_definitions WHERE code = ANY($1) AND is_active = true`, metricCodes)
	if err != nil {
		return err
	}
	defer rows.Close()
	metricUnits := make(map[string]string, len(metricCodes))
	for rows.Next() {
		var code, unit string
		if err := rows.Scan(&code, &unit); err != nil {
			return err
		}
		metricUnits[code] = unit
	}
	if err := rows.Err(); err != nil {
		return err
	}
	for _, input := range inputs {
		if unit, exists := metricUnits[input.MetricCode]; !exists || unit != input.Unit {
			return api.ErrPayrollValidation
		}
	}
	return nil
}

func validatePayrollInputEmployments(ctx context.Context, tx pgx.Tx, employmentIDs []uuid.UUID) error {
	rows, err := tx.Query(ctx, `SELECT id FROM employments WHERE id = ANY($1) AND deleted_at IS NULL`, employmentIDs)
	if err != nil {
		return err
	}
	defer rows.Close()
	valid := make(map[uuid.UUID]struct{}, len(employmentIDs))
	for rows.Next() {
		var id uuid.UUID
		if err := rows.Scan(&id); err != nil {
			return err
		}
		valid[id] = struct{}{}
	}
	if err := rows.Err(); err != nil {
		return err
	}
	if len(valid) != len(uniqueUUIDs(employmentIDs)) {
		return api.ErrEmploymentNotFound
	}
	return nil
}

func ensurePayrollInputPeriodsUnlocked(ctx context.Context, tx pgx.Tx, employmentIDs []uuid.UUID, year, month int) error {
	var locked bool
	err := tx.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1 FROM payroll_periods p
		JOIN employments e ON e.employer_id = p.employer_id
		WHERE e.id = ANY($1) AND p.period_year = $2 AND p.period_month = $3 AND p.status = 'locked'
	)`, employmentIDs, year, month).Scan(&locked)
	if err != nil {
		return err
	}
	if locked {
		return api.ErrPayrollInputConflict
	}
	return nil
}

func insertPayrollInputRows(ctx context.Context, tx pgx.Tx, batchID uuid.UUID, inputs []api.PayrollInput) error {
	valueArgs := make([]any, 0, len(inputs)*8)
	valueParts := make([]string, 0, len(inputs))
	for index, input := range inputs {
		base := index*8 + 1
		var ref any
		if input.SourceReference != nil {
			ref = *input.SourceReference
		}
		status, validationMessage := payrollInputRowStatus(input.Value)
		valueParts = append(valueParts, fmt.Sprintf("($%d, $%d, (SELECT id FROM payroll_metric_definitions WHERE code = $%d), $%d, $%d, $%d, $%d, $%d)", base, base+1, base+2, base+3, base+4, base+5, base+6, base+7))
		valueArgs = append(valueArgs, batchID, input.EmploymentID, input.MetricCode, input.Value.String(), input.Unit, ref, status, validationMessage)
	}
	_, err := tx.Exec(ctx, `INSERT INTO payroll_inputs (batch_id, employment_id, metric_definition_id, value, unit, source_reference, status, validation_message) VALUES `+strings.Join(valueParts, ", "), valueArgs...)
	return err
}

func createPayrollInputBatchTx(ctx context.Context, tx pgx.Tx, req api.CreatePayrollInputRequest, actorID ids.UserID, requestHash string) (api.PayrollInputBatch, error) {
	existing, found, err := findIdempotentPayrollInputBatch(ctx, tx, req.IdempotencyKey, requestHash)
	if err != nil || found {
		return existing, err
	}

	metricCodes := make([]string, 0, len(req.Inputs))
	employmentIDs := make([]uuid.UUID, 0, len(req.Inputs))
	for _, input := range req.Inputs {
		metricCodes = append(metricCodes, input.MetricCode)
		employmentIDs = append(employmentIDs, input.EmploymentID)
	}
	if err := validatePayrollInputMetrics(ctx, tx, req.Inputs, metricCodes); err != nil {
		return api.PayrollInputBatch{}, err
	}
	if err := validatePayrollInputEmployments(ctx, tx, employmentIDs); err != nil {
		return api.PayrollInputBatch{}, err
	}
	if err := ensurePayrollInputPeriodsUnlocked(ctx, tx, employmentIDs, req.PeriodYear, req.PeriodMonth); err != nil {
		return api.PayrollInputBatch{}, err
	}

	var result api.PayrollInputBatch
	batchID := uuid.New()
	if err := tx.QueryRow(ctx, `INSERT INTO payroll_input_batches (id, period_year, period_month, source, idempotency_key, request_hash, status, submitted_by)
		VALUES ($1, $2, $3, $4, $5, $6, 'submitted', $7)
		RETURNING id, period_year, period_month, source, idempotency_key, status, 0, submitted_by, approved_by, locked_by, created_at`, batchID, req.PeriodYear, req.PeriodMonth, strings.TrimSpace(req.Source), req.IdempotencyKey, requestHash, uuid.UUID(actorID)).Scan(
		&result.ID, &result.PeriodYear, &result.PeriodMonth, &result.Source, &result.IdempotencyKey, &result.Status, &result.InputCount, &result.SubmittedBy, &result.ApprovedBy, &result.LockedBy, &result.CreatedAt); err != nil {
		if isConflictError(err) {
			return api.PayrollInputBatch{}, api.ErrPayrollInputConflict
		}
		return api.PayrollInputBatch{}, err
	}
	if err := insertPayrollInputRows(ctx, tx, batchID, req.Inputs); err != nil {
		return api.PayrollInputBatch{}, err
	}
	result.InputCount = len(req.Inputs)
	return result, nil
}

func (r PayrollRepository) CreatePayrollInputBatch(ctx context.Context, req api.CreatePayrollInputRequest, actorID ids.UserID) (api.PayrollInputBatch, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollInputBatch{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.PayrollInputBatch{}, errors.New("payroll repository: database does not support transactions")
	}
	requestHash, err := payrollInputRequestHash(req)
	if err != nil {
		return api.PayrollInputBatch{}, err
	}

	var result api.PayrollInputBatch
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		batch, err := createPayrollInputBatchTx(ctx, tx, req, actorID, requestHash)
		if err != nil {
			return err
		}
		result = batch
		return nil
	})
	if err != nil {
		return api.PayrollInputBatch{}, err
	}
	return result, nil
}

func payrollInputRowStatus(value decimal.Decimal) (string, any) {
	if value.IsZero() {
		return "warning", "zero metric value requires review"
	}
	return "accepted", nil
}

func scanPayrollInputBatch(row interface{ Scan(...any) error }) (api.PayrollInputBatch, error) {
	var batch api.PayrollInputBatch
	var created time.Time
	if err := row.Scan(&batch.ID, &batch.PeriodYear, &batch.PeriodMonth, &batch.Source, &batch.IdempotencyKey, &batch.Status, &batch.InputCount, &batch.SubmittedBy, &batch.ApprovedBy, &batch.LockedBy, &created); err != nil {
		return api.PayrollInputBatch{}, err
	}
	batch.CreatedAt = created.UTC().Format(time.RFC3339)
	return batch, nil
}

func payrollInputRequestHash(req api.CreatePayrollInputRequest) (string, error) {
	type hashInput struct {
		EmploymentID uuid.UUID       `json:"employment_id"`
		MetricCode   string          `json:"metric_code"`
		Value        decimal.Decimal `json:"value"`
		Unit         string          `json:"unit"`
		Reference    *string         `json:"source_reference,omitempty"`
	}
	inputs := make([]hashInput, 0, len(req.Inputs))
	for _, input := range req.Inputs {
		inputs = append(inputs, hashInput{input.EmploymentID, input.MetricCode, input.Value, input.Unit, input.SourceReference})
	}
	sort.Slice(inputs, func(i, j int) bool {
		if inputs[i].EmploymentID != inputs[j].EmploymentID {
			return inputs[i].EmploymentID.String() < inputs[j].EmploymentID.String()
		}
		return inputs[i].MetricCode < inputs[j].MetricCode
	})
	payload, err := json.Marshal(struct {
		Year   int         `json:"year"`
		Month  int         `json:"month"`
		Source string      `json:"source"`
		Inputs []hashInput `json:"inputs"`
	}{req.PeriodYear, req.PeriodMonth, strings.TrimSpace(req.Source), inputs})
	if err != nil {
		return "", err
	}
	return fmt.Sprintf("%x", sha256.Sum256(payload)), nil
}

func (r PayrollRepository) ListPayrollInputs(ctx context.Context, batchID ids.PayrollInputBatchID) ([]api.PayrollInput, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT i.id, i.employment_id, m.code, i.value::text, i.unit, i.status, i.validation_message, i.source_reference
FROM payroll_inputs i JOIN payroll_metric_definitions m ON m.id = i.metric_definition_id
WHERE i.batch_id = $1 ORDER BY i.id ASC`, uuid.UUID(batchID))
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.PayrollInput, 0)
	for rows.Next() {
		var input api.PayrollInput
		var raw string
		if err := rows.Scan(&input.ID, &input.EmploymentID, &input.MetricCode, &raw, &input.Unit, &input.Status, &input.ValidationMessage, &input.SourceReference); err != nil {
			return nil, err
		}
		input.Value, err = decimal.NewFromString(raw)
		if err != nil {
			return nil, err
		}
		result = append(result, input)
	}
	return result, rows.Err()
}

func (r PayrollRepository) CorrectPayrollInput(ctx context.Context, inputID ids.PayrollInputID, req api.CorrectPayrollInputRequest, actorID ids.UserID) (api.PayrollInput, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollInput{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.PayrollInput{}, errors.New("payroll repository: database does not support transactions")
	}
	var result api.PayrollInput
	var correctedValue string
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var oldValue, oldUnit, oldStatus string
		var batchStatus string
		if err := tx.QueryRow(ctx, `SELECT i.value::text, i.unit, i.status, b.status FROM payroll_inputs i JOIN payroll_input_batches b ON b.id = i.batch_id WHERE i.id = $1 FOR UPDATE`, uuid.UUID(inputID)).Scan(&oldValue, &oldUnit, &oldStatus, &batchStatus); err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrPayrollInputNotFound
			}
			return err
		}
		if batchStatus == "locked" {
			return api.ErrPayrollInputConflict
		}
		var oldDecimal decimal.Decimal
		oldDecimal, err = decimal.NewFromString(oldValue)
		if err != nil {
			return err
		}
		if _, err := tx.Exec(ctx, `INSERT INTO payroll_input_corrections (input_id, old_value, new_value, old_unit, new_unit, reason, corrected_by) VALUES ($1, $2, $3, $4, $5, $6, $7)`, uuid.UUID(inputID), oldDecimal.String(), req.Value.String(), oldUnit, strings.TrimSpace(req.Unit), strings.TrimSpace(req.Reason), uuid.UUID(actorID)); err != nil {
			return err
		}
		return tx.QueryRow(ctx, `UPDATE payroll_inputs SET value = $1, unit = $2, status = 'corrected', validation_message = NULL, corrected_at = CURRENT_TIMESTAMP, corrected_by = $3 WHERE id = $4 RETURNING id, employment_id, (SELECT code FROM payroll_metric_definitions WHERE id = metric_definition_id), value::text, unit, status, validation_message, source_reference`, req.Value.String(), strings.TrimSpace(req.Unit), uuid.UUID(actorID), uuid.UUID(inputID)).Scan(&result.ID, &result.EmploymentID, &result.MetricCode, &correctedValue, &result.Unit, &result.Status, &result.ValidationMessage, &result.SourceReference)
	})
	if err != nil {
		return api.PayrollInput{}, err
	}
	result.Value, err = decimal.NewFromString(correctedValue)
	return result, err
}

func (r PayrollRepository) transitionPayrollInputBatch(ctx context.Context, batchID ids.PayrollInputBatchID, actorID ids.UserID, fromStatus, toStatus string) (api.PayrollInputBatch, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollInputBatch{}, err
	}
	var batch api.PayrollInputBatch
	var created time.Time
	query := `UPDATE payroll_input_batches
	SET status = $1, approved_by = CASE WHEN $1 = 'approved' THEN $2 ELSE approved_by END,
	    locked_by = CASE WHEN $1 = 'locked' THEN $2 ELSE locked_by END, updated_at = CURRENT_TIMESTAMP
	WHERE id = $3 AND status = $4
	RETURNING id, period_year, period_month, source, idempotency_key, status, 0, submitted_by, approved_by, locked_by, created_at`
	if err := db.QueryRow(ctx, query, toStatus, uuid.UUID(actorID), uuid.UUID(batchID), fromStatus).Scan(
		&batch.ID, &batch.PeriodYear, &batch.PeriodMonth, &batch.Source, &batch.IdempotencyKey, &batch.Status, &batch.InputCount, &batch.SubmittedBy, &batch.ApprovedBy, &batch.LockedBy, &created); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			var exists bool
			if existsErr := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_input_batches WHERE id = $1)`, batchID).Scan(&exists); existsErr != nil {
				return api.PayrollInputBatch{}, existsErr
			}
			if !exists {
				return api.PayrollInputBatch{}, api.ErrPayrollInputNotFound
			}
			return api.PayrollInputBatch{}, api.ErrPayrollInputConflict
		}
		return api.PayrollInputBatch{}, err
	}
	batch.CreatedAt = created.UTC().Format(time.RFC3339)
	return batch, nil
}

func (r PayrollRepository) ApprovePayrollInputBatch(ctx context.Context, batchID ids.PayrollInputBatchID, actorID ids.UserID) (api.PayrollInputBatch, error) {
	return r.transitionPayrollInputBatch(ctx, batchID, actorID, "submitted", "approved")
}

func (r PayrollRepository) LockPayrollInputBatch(ctx context.Context, batchID ids.PayrollInputBatchID, actorID ids.UserID) (api.PayrollInputBatch, error) {
	return r.transitionPayrollInputBatch(ctx, batchID, actorID, "approved", "locked")
}

func uniqueUUIDs(ids []uuid.UUID) map[uuid.UUID]struct{} {
	unique := make(map[uuid.UUID]struct{}, len(ids))
	for _, id := range ids {
		unique[id] = struct{}{}
	}
	return unique
}
