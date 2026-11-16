package repository

import (
	"context"
	"errors"
	"fmt"
	"sort"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type ShiftSwapRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r ShiftSwapRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("shift swap repository: database is not configured")
	}
	return r.DB, nil
}

func (r ShiftSwapRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("shift swap repository: database does not support transactions")
	}
	return beginner, nil
}

func (r ShiftSwapRepository) CreateSwap(ctx context.Context, userID uuid.UUID, req api.CreateShiftSwapRequest) (api.ShiftSwapRequest, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.ShiftSwapRequest{}, err
	}
	id := uuid.New()
	now := time.Now().UTC()
	var result api.ShiftSwapRequest
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		reqEmployment, err := resolveActiveEmployment(ctx, db, req.RequesterID)
		if err != nil {
			return api.ErrShiftSwapValidation
		}
		tgtEmployment, err := resolveActiveEmployment(ctx, db, req.TargetID)
		if err != nil {
			return api.ErrShiftSwapValidation
		}
		var activeSwapExists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM shift_swap_requests WHERE requester_id = $1 AND target_id = $2 AND swap_date = $3::date AND status NOT IN ('cancelled', 'rejected'))`, req.RequesterID, req.TargetID, req.SwapDate).Scan(&activeSwapExists); err != nil {
			return err
		}
		if activeSwapExists {
			return api.ErrShiftSwapConflict
		}
		var conflictExists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM shift_swap_requests WHERE (requester_id IN ($1, $2) OR target_id IN ($1, $2)) AND swap_date = $3::date AND status = 'approved')`, req.RequesterID, req.TargetID, req.SwapDate).Scan(&conflictExists); err != nil {
			return err
		}
		if conflictExists {
			return api.ErrShiftSwapConflict
		}
		reqShift, err := lookupShiftOnDate(ctx, db, reqEmployment, req.SwapDate)
		if err != nil {
			return err
		}
		tgtShift, err := lookupShiftOnDate(ctx, db, tgtEmployment, req.SwapDate)
		if err != nil {
			return err
		}
		if sameShift(reqShift, tgtShift) {
			return api.ErrShiftSwapValidation
		}
		requesterRevisionID, requesterRevisionNumber, err := currentOfficialRevision(ctx, db, reqEmployment, req.SwapDate)
		if err != nil {
			return err
		}
		targetRevisionID, targetRevisionNumber, err := currentOfficialRevision(ctx, db, tgtEmployment, req.SwapDate)
		if err != nil {
			return err
		}
		_, err = db.Exec(ctx, `INSERT INTO shift_swap_requests (id, requester_id, target_id, swap_date, requester_template_id, target_template_id, requester_revision_id, requester_revision_number, target_revision_id, target_revision_number, status, reason, requested_by_user_id, approved_by_user_id, created_at, updated_at) VALUES ($1, $2, $3, $4::date, $5, $6, $7, $8, $9, $10, 'pending_admin', $11, $12, NULL, $13, $13)`, id, req.RequesterID, req.TargetID, req.SwapDate, reqShift, tgtShift, requesterRevisionID, requesterRevisionNumber, targetRevisionID, targetRevisionNumber, req.Reason, userID, now)
		if err != nil {
			return err
		}
		result = api.ShiftSwapRequest{
			ID:                      id,
			RequesterID:             req.RequesterID,
			TargetID:                req.TargetID,
			SwapDate:                req.SwapDate,
			RequesterRevisionID:     requesterRevisionID,
			RequesterRevisionNumber: requesterRevisionNumber,
			TargetRevisionID:        targetRevisionID,
			TargetRevisionNumber:    targetRevisionNumber,
			RequesterShiftID:        reqShift,
			TargetShiftID:           tgtShift,
			Status:                  api.ShiftSwapPendingAdmin,
			Reason:                  req.Reason,
			RequestedByUserID:       &userID,
			CreatedAt:               now.Format(time.RFC3339),
			UpdatedAt:               now.Format(time.RFC3339),
		}
		return nil
	})
	if err != nil {
		return api.ShiftSwapRequest{}, err
	}
	return result, nil
}

func (r ShiftSwapRepository) ListSwaps(ctx context.Context, status *api.ShiftSwapStatus) ([]api.ShiftSwapRequestWithNames, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	var rows pgx.Rows
	if status != nil {
		rows, err = db.Query(ctx, `SELECT id, requester_id, target_id, swap_date::text, requester_revision_id, requester_revision_number, target_revision_id, target_revision_number, requester_template_id, target_template_id, status, reason, requested_by_user_id, approved_by_user_id, created_at, updated_at FROM shift_swap_requests WHERE status = $1 ORDER BY created_at DESC LIMIT 100`, string(*status))
	} else {
		rows, err = db.Query(ctx, `SELECT id, requester_id, target_id, swap_date::text, requester_revision_id, requester_revision_number, target_revision_id, target_revision_number, requester_template_id, target_template_id, status, reason, requested_by_user_id, approved_by_user_id, created_at, updated_at FROM shift_swap_requests ORDER BY created_at DESC LIMIT 100`)
	}
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	swaps := make([]api.ShiftSwapRequestWithNames, 0)
	empIDs := make([]uuid.UUID, 0)
	shiftIDs := make([]uuid.UUID, 0)
	for rows.Next() {
		var s api.ShiftSwapRequestWithNames
		var statusStr string
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&s.ID, &s.RequesterID, &s.TargetID, &s.SwapDate, &s.RequesterRevisionID, &s.RequesterRevisionNumber, &s.TargetRevisionID, &s.TargetRevisionNumber, &s.RequesterShiftID, &s.TargetShiftID, &statusStr, &s.Reason, &s.RequestedByUserID, &s.ApprovedByUserID, &createdAt, &updatedAt); err != nil {
			return nil, err
		}
		s.Status = api.ShiftSwapStatus(statusStr)
		s.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		s.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		empIDs = append(empIDs, s.RequesterID, s.TargetID)
		if s.RequesterShiftID != nil {
			shiftIDs = append(shiftIDs, *s.RequesterShiftID)
		}
		if s.TargetShiftID != nil {
			shiftIDs = append(shiftIDs, *s.TargetShiftID)
		}
		swaps = append(swaps, s)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	empNames, err := batchLoadEmployeeNames(ctx, db, empIDs)
	if err != nil {
		return nil, err
	}
	shiftNames, err := batchLoadShiftNames(ctx, db, shiftIDs)
	if err != nil {
		return nil, err
	}
	for index := range swaps {
		if name, ok := empNames[swaps[index].RequesterID]; ok {
			swaps[index].RequesterName = &name
		}
		if name, ok := empNames[swaps[index].TargetID]; ok {
			swaps[index].TargetName = &name
		}
		if swaps[index].RequesterShiftID != nil {
			if name, ok := shiftNames[*swaps[index].RequesterShiftID]; ok {
				swaps[index].RequesterShiftName = &name
			}
		}
		if swaps[index].TargetShiftID != nil {
			if name, ok := shiftNames[*swaps[index].TargetShiftID]; ok {
				swaps[index].TargetShiftName = &name
			}
		}
	}
	return swaps, nil
}

func (r ShiftSwapRepository) ApproveSwap(ctx context.Context, requestID, approverID uuid.UUID, newStatus api.ShiftSwapStatus) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	now := time.Now().UTC()
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var currentStatus string
		var requesterID, targetID uuid.UUID
		var swapDate string
		var reqShift, tgtShift *uuid.UUID
		var requesterRevisionID, targetRevisionID *uuid.UUID
		var requesterRevisionNumber, targetRevisionNumber *int64
		err := db.QueryRow(ctx, `SELECT status, requester_id, target_id, swap_date::text, requester_template_id, target_template_id, requester_revision_id, requester_revision_number, target_revision_id, target_revision_number FROM shift_swap_requests WHERE id = $1`, requestID).Scan(&currentStatus, &requesterID, &targetID, &swapDate, &reqShift, &tgtShift, &requesterRevisionID, &requesterRevisionNumber, &targetRevisionID, &targetRevisionNumber)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrShiftSwapNotFound
		}
		if err != nil {
			return err
		}
		if currentStatus != string(api.ShiftSwapPendingAdmin) {
			return api.ErrShiftSwapConflict
		}
		_, err = db.Exec(ctx, `UPDATE shift_swap_requests SET status = $1, approved_by_user_id = $2, updated_at = $3 WHERE id = $4`, string(newStatus), approverID, now, requestID)
		if err != nil {
			return err
		}
		if newStatus == api.ShiftSwapApproved {
			reqEmp, err := resolveEmploymentOnDate(ctx, db, requesterID, swapDate)
			if err != nil {
				return err
			}
			tgtEmp, err := resolveEmploymentOnDate(ctx, db, targetID, swapDate)
			if err != nil {
				return err
			}
			if requesterRevisionID != nil || targetRevisionID != nil {
				if requesterRevisionID == nil || targetRevisionID == nil || requesterRevisionNumber == nil || targetRevisionNumber == nil {
					return api.ErrShiftSwapConflict
				}
				handled, err := approveOfficialSwap(ctx, db, requestID, approverID, requesterID, targetID, reqEmp, tgtEmp, swapDate, *requesterRevisionID, *requesterRevisionNumber, *targetRevisionID, *targetRevisionNumber)
				if err != nil {
					return err
				}
				if handled {
					return nil
				}
			}
			if err := guardLifecycleDateMutation(ctx, db, []uuid.UUID{reqEmp, tgtEmp}, []string{swapDate}); err != nil {
				if errors.Is(err, ErrLifecycleMutationBlocked) || errors.Is(err, ErrLifecycleSchemaUnavailable) {
					return api.ErrShiftSwapConflict
				}
				return err
			}
			note := fmt.Sprintf("Shift swap #%s", requestID)
			if _, err := db.Exec(ctx, `INSERT INTO shift_rosters (id, employment_id, template_id, date, status, notes, created_at, updated_at) VALUES ($1, $2, $3, $4::date, 'active', $5, $6, $6) ON CONFLICT (employment_id, date) DO UPDATE SET template_id = EXCLUDED.template_id, status = EXCLUDED.status, notes = EXCLUDED.notes, updated_at = EXCLUDED.updated_at`, uuid.New(), reqEmp, tgtShift, swapDate, note, now); err != nil {
				return err
			}
			if _, err := db.Exec(ctx, `INSERT INTO shift_rosters (id, employment_id, template_id, date, status, notes, created_at, updated_at) VALUES ($1, $2, $3, $4::date, 'active', $5, $6, $6) ON CONFLICT (employment_id, date) DO UPDATE SET template_id = EXCLUDED.template_id, status = EXCLUDED.status, notes = EXCLUDED.notes, updated_at = EXCLUDED.updated_at`, uuid.New(), tgtEmp, reqShift, swapDate, note, now); err != nil {
				return err
			}
			if err := EnqueueRosterCacheInvalidation(ctx, db); err != nil {
				return err
			}
		}
		return nil
	})
}

func (r ShiftSwapRepository) Stats(ctx context.Context) (api.SwapStats, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.SwapStats{}, err
	}
	var stats api.SwapStats
	err = db.QueryRow(ctx, `SELECT COALESCE(SUM(CASE WHEN status = 'pending_target' THEN 1 ELSE 0 END), 0), COALESCE(SUM(CASE WHEN status = 'pending_admin' THEN 1 ELSE 0 END), 0), COALESCE(SUM(CASE WHEN status = 'approved' THEN 1 ELSE 0 END), 0), COALESCE(SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END), 0), COALESCE(SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END), 0), COALESCE(SUM(CASE WHEN status = 'approved' AND created_at >= date_trunc('month', CURRENT_DATE) THEN 1 ELSE 0 END), 0), COALESCE(SUM(CASE WHEN status = 'rejected' AND created_at >= date_trunc('month', CURRENT_DATE) THEN 1 ELSE 0 END), 0) FROM shift_swap_requests`).Scan(&stats.PendingTarget, &stats.PendingAdmin, &stats.Approved, &stats.Rejected, &stats.Cancelled, &stats.ApprovedThisMonth, &stats.RejectedThisMonth)
	return stats, err
}

func (r ShiftSwapRepository) ListEmployeeSwaps(ctx context.Context, employeeID uuid.UUID) ([]api.ShiftSwapRequestWithNames, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT s.id, s.requester_id, s.target_id, s.swap_date::text, s.requester_revision_id, s.requester_revision_number, s.target_revision_id, s.target_revision_number, s.requester_template_id, s.target_template_id, s.status, s.reason, s.requested_by_user_id, s.approved_by_user_id, s.created_at, s.updated_at FROM shift_swap_requests s WHERE (s.requester_id = $1 OR s.target_id = $1) AND EXISTS (SELECT 1 FROM employments em JOIN employment_relationships er ON er.id = em.relationship_id AND er.employee_id = em.employee_id JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = er.employee_id WHERE em.employee_id = $1 AND em.deleted_at IS NULL AND t.deleted_at IS NULL AND t.start_date <= CURRENT_DATE AND (t.end_date IS NULL OR t.end_date >= CURRENT_DATE)) ORDER BY created_at DESC LIMIT 50`, employeeID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	swaps := make([]api.ShiftSwapRequestWithNames, 0)
	empIDs := make([]uuid.UUID, 0)
	shiftIDs := make([]uuid.UUID, 0)
	for rows.Next() {
		var s api.ShiftSwapRequestWithNames
		var statusStr string
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&s.ID, &s.RequesterID, &s.TargetID, &s.SwapDate, &s.RequesterRevisionID, &s.RequesterRevisionNumber, &s.TargetRevisionID, &s.TargetRevisionNumber, &s.RequesterShiftID, &s.TargetShiftID, &statusStr, &s.Reason, &s.RequestedByUserID, &s.ApprovedByUserID, &createdAt, &updatedAt); err != nil {
			return nil, err
		}
		s.Status = api.ShiftSwapStatus(statusStr)
		s.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		s.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		empIDs = append(empIDs, s.RequesterID, s.TargetID)
		if s.RequesterShiftID != nil {
			shiftIDs = append(shiftIDs, *s.RequesterShiftID)
		}
		if s.TargetShiftID != nil {
			shiftIDs = append(shiftIDs, *s.TargetShiftID)
		}
		swaps = append(swaps, s)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	empNames, err := batchLoadEmployeeNames(ctx, db, empIDs)
	if err != nil {
		return nil, err
	}
	shiftNames, err := batchLoadShiftNames(ctx, db, shiftIDs)
	if err != nil {
		return nil, err
	}
	for index := range swaps {
		if name, ok := empNames[swaps[index].RequesterID]; ok {
			swaps[index].RequesterName = &name
		}
		if name, ok := empNames[swaps[index].TargetID]; ok {
			swaps[index].TargetName = &name
		}
		if swaps[index].RequesterShiftID != nil {
			if name, ok := shiftNames[*swaps[index].RequesterShiftID]; ok {
				swaps[index].RequesterShiftName = &name
			}
		}
		if swaps[index].TargetShiftID != nil {
			if name, ok := shiftNames[*swaps[index].TargetShiftID]; ok {
				swaps[index].TargetShiftName = &name
			}
		}
	}
	return swaps, nil
}

func (r ShiftSwapRepository) CreateESSSwap(ctx context.Context, requesterID, userID uuid.UUID, req api.EssCreateShiftSwapRequest) (api.ShiftSwapRequest, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.ShiftSwapRequest{}, err
	}
	id := uuid.New()
	now := time.Now().UTC()
	var result api.ShiftSwapRequest
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		reqEmployment, err := resolveActiveEmployment(ctx, db, requesterID)
		if err != nil {
			return api.ErrShiftSwapValidation
		}
		tgtEmployment, err := resolveActiveEmployment(ctx, db, req.TargetID)
		if err != nil {
			return api.ErrShiftSwapValidation
		}
		var activeSwapExists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM shift_swap_requests WHERE requester_id = $1 AND target_id = $2 AND swap_date = $3::date AND status NOT IN ('cancelled', 'rejected'))`, requesterID, req.TargetID, req.SwapDate).Scan(&activeSwapExists); err != nil {
			return err
		}
		if activeSwapExists {
			return api.ErrShiftSwapConflict
		}
		var conflictExists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM shift_swap_requests WHERE (requester_id IN ($1, $2) OR target_id IN ($1, $2)) AND swap_date = $3::date AND status = 'approved')`, requesterID, req.TargetID, req.SwapDate).Scan(&conflictExists); err != nil {
			return err
		}
		if conflictExists {
			return api.ErrShiftSwapConflict
		}
		reqShift, err := lookupShiftOnDate(ctx, db, reqEmployment, req.SwapDate)
		if err != nil {
			return err
		}
		tgtShift, err := lookupShiftOnDate(ctx, db, tgtEmployment, req.SwapDate)
		if err != nil {
			return err
		}
		if sameShift(reqShift, tgtShift) {
			return api.ErrShiftSwapValidation
		}
		requesterRevisionID, requesterRevisionNumber, err := currentOfficialRevision(ctx, db, reqEmployment, req.SwapDate)
		if err != nil {
			return err
		}
		targetRevisionID, targetRevisionNumber, err := currentOfficialRevision(ctx, db, tgtEmployment, req.SwapDate)
		if err != nil {
			return err
		}
		_, err = db.Exec(ctx, `INSERT INTO shift_swap_requests (id, requester_id, target_id, swap_date, requester_template_id, target_template_id, requester_revision_id, requester_revision_number, target_revision_id, target_revision_number, status, reason, requested_by_user_id, approved_by_user_id, created_at, updated_at) VALUES ($1, $2, $3, $4::date, $5, $6, $7, $8, $9, $10, 'pending_target', $11, $12, NULL, $13, $13)`, id, requesterID, req.TargetID, req.SwapDate, reqShift, tgtShift, requesterRevisionID, requesterRevisionNumber, targetRevisionID, targetRevisionNumber, req.Reason, userID, now)
		if err != nil {
			return err
		}
		result = api.ShiftSwapRequest{
			ID:                      id,
			RequesterID:             requesterID,
			TargetID:                req.TargetID,
			SwapDate:                req.SwapDate,
			RequesterRevisionID:     requesterRevisionID,
			RequesterRevisionNumber: requesterRevisionNumber,
			TargetRevisionID:        targetRevisionID,
			TargetRevisionNumber:    targetRevisionNumber,
			RequesterShiftID:        reqShift,
			TargetShiftID:           tgtShift,
			Status:                  api.ShiftSwapPendingTarget,
			Reason:                  req.Reason,
			RequestedByUserID:       &userID,
			CreatedAt:               now.Format(time.RFC3339),
			UpdatedAt:               now.Format(time.RFC3339),
		}
		return nil
	})
	if err != nil {
		return api.ShiftSwapRequest{}, err
	}
	return result, nil
}

func (r ShiftSwapRepository) RespondSwap(ctx context.Context, targetID, requestID uuid.UUID, accept bool) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	now := time.Now().UTC()
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var currentStatus string
		var swapTargetID uuid.UUID
		err := db.QueryRow(ctx, `SELECT status, target_id FROM shift_swap_requests WHERE id = $1`, requestID).Scan(&currentStatus, &swapTargetID)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrShiftSwapNotFound
		}
		if err != nil {
			return err
		}
		if swapTargetID != targetID {
			return api.ErrShiftSwapForbidden
		}
		if currentStatus != string(api.ShiftSwapPendingTarget) {
			return api.ErrShiftSwapConflict
		}
		newStatus := api.ShiftSwapPendingAdmin
		if !accept {
			newStatus = api.ShiftSwapRejected
		}
		_, err = db.Exec(ctx, `UPDATE shift_swap_requests SET status = $1, updated_at = $2 WHERE id = $3`, string(newStatus), now, requestID)
		return err
	})
}

func (r ShiftSwapRepository) CancelSwap(ctx context.Context, requesterID, requestID uuid.UUID) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	now := time.Now().UTC()
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var currentStatus string
		var swapRequesterID uuid.UUID
		err := db.QueryRow(ctx, `SELECT status, requester_id FROM shift_swap_requests WHERE id = $1`, requestID).Scan(&currentStatus, &swapRequesterID)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrShiftSwapNotFound
		}
		if err != nil {
			return err
		}
		if swapRequesterID != requesterID {
			return api.ErrShiftSwapForbidden
		}
		if currentStatus != string(api.ShiftSwapPendingTarget) && currentStatus != string(api.ShiftSwapPendingAdmin) {
			return api.ErrShiftSwapConflict
		}
		_, err = db.Exec(ctx, `UPDATE shift_swap_requests SET status = 'cancelled', updated_at = $1 WHERE id = $2`, now, requestID)
		return err
	})
}

func approveOfficialSwap(ctx context.Context, db pgx.Tx, requestID, approverID, requesterID, targetID, requesterEmploymentID, targetEmploymentID uuid.UUID, date string, requesterRevisionID uuid.UUID, requesterRevisionNumber int64, targetRevisionID uuid.UUID, targetRevisionNumber int64) (bool, error) {
	if requesterRevisionID == uuid.Nil || targetRevisionID == uuid.Nil {
		return false, nil
	}
	employees := []uuid.UUID{requesterID, targetID}
	sort.Slice(employees, func(i, j int) bool { return employees[i].String() < employees[j].String() })
	if _, err := db.Exec(ctx, `SELECT id FROM employees WHERE id = ANY($1) ORDER BY id FOR UPDATE`, employees); err != nil {
		return false, err
	}
	employments := []uuid.UUID{requesterEmploymentID, targetEmploymentID}
	sort.Slice(employments, func(i, j int) bool { return employments[i].String() < employments[j].String() })
	if _, err := db.Exec(ctx, `SELECT id FROM employments WHERE id = ANY($1) ORDER BY id FOR UPDATE`, employments); err != nil {
		return false, err
	}
	requester, err := loadCurrentRevision(ctx, db, requesterEmploymentID, date, true)
	if err != nil {
		return false, err
	}
	target, err := loadCurrentRevision(ctx, db, targetEmploymentID, date, true)
	if err != nil {
		return false, err
	}
	if requester.ID != requesterRevisionID || requester.RevisionNumber != requesterRevisionNumber || target.ID != targetRevisionID || target.RevisionNumber != targetRevisionNumber {
		return false, api.ErrShiftSwapConflict
	}
	if err := ensureMutationPeriodOpen(ctx, db, requester.EmployerID, date); err != nil {
		return false, err
	}
	if err := ensureMutationPeriodOpen(ctx, db, target.EmployerID, date); err != nil {
		return false, err
	}
	requesterDoc, err := loadRevisionDocument(ctx, db, requester)
	if err != nil {
		return false, err
	}
	targetDoc, err := loadRevisionDocument(ctx, db, target)
	if err != nil {
		return false, err
	}
	requesterDoc["employment_id"], targetDoc["employment_id"] = requesterEmploymentID.String(), targetEmploymentID.String()
	replaceShiftVersion(requesterDoc, targetDoc)
	replaceShiftVersion(targetDoc, requesterDoc)
	batchID, now := uuid.New(), time.Now().UTC()
	key := "shift-swap:" + requestID.String()
	hash, err := hashJSON(map[string]any{"request_id": requestID.String(), "requester_revision": requesterRevisionID.String(), "target_revision": targetRevisionID.String()})
	if err != nil {
		return false, err
	}
	reason := "Shift swap #" + requestID.String()
	if _, err := db.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, actor_type, reason, timezone, range_start, range_end, published_at, affected_days) VALUES ($1, $2, $3, 'rotating', 'published', $4, 'replan', '{}'::jsonb, 'human', $5, $6, $7::date, $7::date, $8, 2)`, batchID, key, hash, approverID, reason, requester.Timezone, date, now); err != nil {
		return false, err
	}
	if _, err := db.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
		return false, err
	}
	if _, err := db.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'superseded' WHERE id IN ($1, $2) AND state IN ('published', 'explicit_off')`, requester.ID, target.ID); err != nil {
		return false, err
	}
	newRequester, err := insertPublishedDay(ctx, db, batchID, publicationActor{ID: &approverID, Type: "human"}, reason, requester.Timezone, requesterDoc, now, &requester.ID)
	if err != nil {
		return false, err
	}
	newTarget, err := insertPublishedDay(ctx, db, batchID, publicationActor{ID: &approverID, Type: "human"}, reason, target.Timezone, targetDoc, now, &target.ID)
	if err != nil {
		return false, err
	}
	response := jsonArg(map[string]any{"request_id": requestID.String(), "revision_ids": []string{newRequester.String(), newTarget.String()}})
	if _, err := db.Exec(ctx, `UPDATE schedule_publication_batches SET response_snapshot = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $1`, batchID, response); err != nil {
		return false, err
	}
	if _, err := db.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, created_at, actor_type) VALUES ($1, $2, 'schedule.swap_replanned', $3, CURRENT_TIMESTAMP, 'human')`, approverID, batchID, response); err != nil {
		return false, err
	}
	return true, nil
}

func replaceShiftVersion(source, donor map[string]any) {
	if intervals, ok := donor["intervals"].([]map[string]any); ok {
		source["intervals"] = intervals
	}
	if value, ok := donor["shift_version_id"]; ok {
		source["shift_version_id"] = value
	}
}

func currentOfficialRevision(ctx context.Context, db DBTX, employmentID uuid.UUID, date string) (*uuid.UUID, *int64, error) {
	var revisionID *uuid.UUID
	var revisionNumber *int64
	if err := db.QueryRow(ctx, `SELECT id, revision_number FROM schedule_day_revisions WHERE employment_id = $1 AND business_date = $2::date AND state IN ('published', 'explicit_off') ORDER BY revision_number DESC LIMIT 1`, employmentID, date).Scan(&revisionID, &revisionNumber); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, nil, nil
		}
		return nil, nil, err
	}
	return revisionID, revisionNumber, nil
}

func resolveEmploymentOnDate(ctx context.Context, db DBTX, employeeID uuid.UUID, date string) (uuid.UUID, error) {
	var employmentID uuid.UUID
	err := db.QueryRow(ctx, `SELECT em.id FROM employments em JOIN employment_relationships er ON er.id = em.relationship_id AND er.employee_id = em.employee_id JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = er.employee_id WHERE em.employee_id = $1 AND em.deleted_at IS NULL AND em.start_date <= $2::date AND (em.end_date IS NULL OR em.end_date >= $2::date) AND er.start_date <= $2::date AND (er.end_date IS NULL OR er.end_date >= $2::date) AND t.deleted_at IS NULL AND t.start_date <= $2::date AND (t.end_date IS NULL OR t.end_date >= $2::date) ORDER BY (em.status = 'active') DESC, em.start_date DESC, em.id DESC LIMIT 1`, employeeID, date).Scan(&employmentID)
	if err != nil {
		return uuid.Nil, api.ErrNoActiveEmployment
	}
	return employmentID, nil
}

func lookupShiftOnDate(ctx context.Context, db DBTX, employmentID uuid.UUID, date string) (*uuid.UUID, error) {
	var shiftID *uuid.UUID
	err := db.QueryRow(ctx, `SELECT template_id FROM shift_rosters WHERE employment_id = $1 AND date = $2::date`, employmentID, date).Scan(&shiftID)
	if err == nil {
		return shiftID, nil
	}
	if !errors.Is(err, pgx.ErrNoRows) {
		return nil, err
	}
	err = db.QueryRow(ctx, `SELECT template_id FROM employee_shifts WHERE employment_id = $1 AND start_date <= $2::date AND (end_date IS NULL OR end_date >= $2::date) ORDER BY start_date DESC LIMIT 1`, employmentID, date).Scan(&shiftID)
	if err == nil {
		return shiftID, nil
	}
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	return nil, err
}

func sameShift(left, right *uuid.UUID) bool {
	if left == nil && right == nil {
		return true
	}
	if left != nil && right != nil && *left == *right {
		return true
	}
	return false
}

func batchLoadEmployeeNames(ctx context.Context, db DBTX, ids []uuid.UUID) (map[uuid.UUID]string, error) {
	if len(ids) == 0 {
		return map[uuid.UUID]string{}, nil
	}
	rows, err := db.Query(ctx, `SELECT id, full_name FROM employees WHERE id = ANY($1)`, ids)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make(map[uuid.UUID]string, len(ids))
	for rows.Next() {
		var id uuid.UUID
		var name string
		if err := rows.Scan(&id, &name); err != nil {
			return nil, err
		}
		result[id] = name
	}
	return result, rows.Err()
}

func batchLoadShiftNames(ctx context.Context, db DBTX, ids []uuid.UUID) (map[uuid.UUID]string, error) {
	if len(ids) == 0 {
		return map[uuid.UUID]string{}, nil
	}
	rows, err := db.Query(ctx, `SELECT id, name FROM shift_templates WHERE id = ANY($1)`, ids)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make(map[uuid.UUID]string, len(ids))
	for rows.Next() {
		var id uuid.UUID
		var name string
		if err := rows.Scan(&id, &name); err != nil {
			return nil, err
		}
		result[id] = name
	}
	return result, rows.Err()
}
