package service

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/external"
)

type PullDevice struct {
	ID         uuid.UUID
	ProviderID string
}
type PullDeviceSource interface {
	ListPullDevices(context.Context, uuid.UUID) ([]PullDevice, error)
}
type AttendanceEventsPageClient interface {
	GetPage(context.Context, string, string, string, string) (external.AttendanceEventsPage, error)
}
type PullRepository interface {
	AcquireLease(context.Context, uuid.UUID, uuid.UUID, string, time.Time) (uuid.UUID, bool, error)
	CurrentCursor(context.Context, uuid.UUID, uuid.UUID) (string, error)
	ReleaseLease(context.Context, uuid.UUID, uuid.UUID, uuid.UUID, error) error
	ReleaseLeaseSuccess(context.Context, uuid.UUID, uuid.UUID, uuid.UUID) error
	WithPageTransaction(context.Context, uuid.UUID, func(context.Context, pgx.Tx) error) error
	RecordEventTx(context.Context, pgx.Tx, uuid.UUID, string, string, time.Time, []byte) (bool, error)
	AdvanceCursorTx(context.Context, pgx.Tx, uuid.UUID, uuid.UUID, uuid.UUID, string) error
}

type ExternalAttendancePullService struct {
	Devices       PullDeviceSource
	Events        AttendanceEventsPageClient
	Pulls         PullRepository
	AccountID     string
	Owner         string
	LeaseDuration time.Duration
	Shadow        bool
	Now           func() time.Time
}

func (s ExternalAttendancePullService) Pull(ctx context.Context, workspaceID uuid.UUID, requestID string) error {
	if s.Devices == nil || s.Events == nil || s.Pulls == nil {
		return errors.New("external attendance pull: dependencies are required")
	}
	if s.AccountID == "" {
		return errors.New("external attendance pull: account ID is required")
	}
	devices, err := s.Devices.ListPullDevices(ctx, workspaceID)
	if err != nil {
		return err
	}
	for _, device := range devices {
		if err := s.pullDevice(ctx, workspaceID, device, requestID); err != nil {
			return err
		}
	}
	return nil
}

func (s ExternalAttendancePullService) PullShadow(ctx context.Context, workspaceID uuid.UUID, requestID string) error {
	shadow := s
	shadow.Shadow = true
	return shadow.Pull(ctx, workspaceID, requestID)
}

func (s ExternalAttendancePullService) pullDevice(ctx context.Context, workspaceID uuid.UUID, device PullDevice, requestID string) error {
	if device.ID == uuid.Nil || strings.TrimSpace(device.ProviderID) == "" {
		return errors.New("external attendance pull: device IDs are required")
	}
	owner := s.Owner
	if owner == "" {
		owner = "external-attendance-pull"
	}
	duration := s.LeaseDuration
	if duration <= 0 {
		duration = 5 * time.Minute
	}
	now := time.Now
	if s.Now != nil {
		now = s.Now
	}
	lease, claimed, err := s.Pulls.AcquireLease(ctx, workspaceID, device.ID, owner, now().Add(duration))
	if err != nil || !claimed {
		return err
	}
	cursor, err := s.Pulls.CurrentCursor(ctx, workspaceID, device.ID)
	if err != nil {
		if s.Shadow {
			_ = s.Pulls.ReleaseLeaseSuccess(ctx, workspaceID, device.ID, lease)
		} else {
			_ = s.Pulls.ReleaseLease(ctx, workspaceID, device.ID, lease, err)
		}
		return err
	}
	for {
		page, pageErr := s.Events.GetPage(ctx, s.AccountID, device.ProviderID, cursor, requestID)
		if pageErr != nil {
			if s.Shadow {
				_ = s.Pulls.ReleaseLeaseSuccess(ctx, workspaceID, device.ID, lease)
			} else {
				_ = s.Pulls.ReleaseLease(ctx, workspaceID, device.ID, lease, pageErr)
			}
			return pageErr
		}
		if s.Shadow {
			for _, event := range page.Events {
				if _, err := NormalizeAttendanceEvent(device.ProviderID, event); err != nil {
					_ = s.Pulls.ReleaseLeaseSuccess(ctx, workspaceID, device.ID, lease)
					return err
				}
			}
			if page.NextCursor == "" {
				return s.Pulls.ReleaseLeaseSuccess(ctx, workspaceID, device.ID, lease)
			}
			cursor = page.NextCursor
			continue
		}
		err = s.Pulls.WithPageTransaction(ctx, workspaceID, func(txCtx context.Context, tx pgx.Tx) error {
			for _, event := range page.Events {
				if err := s.persistEvent(txCtx, tx, workspaceID, device.ProviderID, event); err != nil {
					return err
				}
			}
			return s.Pulls.AdvanceCursorTx(txCtx, tx, workspaceID, device.ID, lease, page.NextCursor)
		})
		if err != nil {
			_ = s.Pulls.ReleaseLease(ctx, workspaceID, device.ID, lease, err)
			return err
		}
		if page.NextCursor == "" {
			return nil
		}
		cursor = page.NextCursor
	}
}
func (s ExternalAttendancePullService) persistEvent(ctx context.Context, tx pgx.Tx, workspaceID uuid.UUID, providerID string, event external.AttendanceEvent) error {
	normalized, err := NormalizeAttendanceEvent(providerID, event)
	if err != nil {
		return err
	}
	metadata, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("external attendance pull: encode event metadata: %w", err)
	}
	inserted, err := s.Pulls.RecordEventTx(ctx, tx, workspaceID, providerID, normalized.ExternalEventID, normalized.OccurredAt, metadata)
	if err != nil || !inserted {
		return err
	}
	var employmentID uuid.UUID
	err = tx.QueryRow(ctx, `SELECT em.id FROM employments em JOIN employees e ON e.id = em.employee_id WHERE em.workspace_id = $1 AND UPPER(e.employee_id_number) = UPPER($2) AND em.status = 'active' AND em.deleted_at IS NULL AND e.deleted_at IS NULL LIMIT 1`, workspaceID, normalized.EmployeeCode).Scan(&employmentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return stageExternalAttendance(ctx, tx, workspaceID, providerID, normalized)
	}
	if err != nil {
		return err
	}
	_, err = tx.Exec(ctx, `INSERT INTO raw_attendance_logs (id, workspace_id, employment_id, punch_time, source, device_id, raw_data, punch, status, created_at) VALUES ($1, $2, $3, $4, 'import', $5, $6, $7, 0, NOW()) ON CONFLICT (workspace_id, employment_id, punch_time, punch, status) DO NOTHING`, uuid.New(), workspaceID, employmentID, normalized.OccurredAt, providerID, metadata, normalized.Punch)
	return err
}

func stageExternalAttendance(ctx context.Context, tx pgx.Tx, workspaceID uuid.UUID, providerID string, event NormalizedAttendanceEvent) error {
	raw, err := json.Marshal(map[string]any{"employee_code": event.EmployeeCode, "external_event_id": event.ExternalEventID, "device_id": providerID, "provider_data": json.RawMessage(event.Data)})
	if err != nil {
		return err
	}
	fingerprint := providerID + "|" + event.ExternalEventID
	_, err = tx.Exec(ctx, `INSERT INTO attendance_staging_logs (workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint, status, created_at) VALUES ($1, $2, $3, $4, $5, $6, 'pending', NOW())`, workspaceID, providerID, event.EmployeeCode, event.OccurredAt, raw, fingerprint)
	return err
}

type NormalizedAttendanceEvent struct {
	ExternalEventID, EmployeeCode, Data string
	OccurredAt                          time.Time
	Punch                               int
}

func NormalizeAttendanceEvent(providerID string, event external.AttendanceEvent) (NormalizedAttendanceEvent, error) {
	if strings.TrimSpace(event.ExternalEventID) == "" {
		return NormalizedAttendanceEvent{}, errors.New("external attendance pull: event ID is required")
	}
	if strings.TrimSpace(providerID) == "" || strings.TrimSpace(event.DeviceID) != strings.TrimSpace(providerID) {
		return NormalizedAttendanceEvent{}, errors.New("external attendance pull: event device does not match configured device")
	}
	occurred, err := time.Parse(time.RFC3339Nano, strings.TrimSpace(event.OccurredAt))
	if err != nil {
		return NormalizedAttendanceEvent{}, errors.New("external attendance pull: event occurred_at is invalid")
	}
	var data struct {
		EmployeeCode string `json:"employee_code"`
	}
	if len(event.Data) == 0 || json.Unmarshal(event.Data, &data) != nil || strings.TrimSpace(data.EmployeeCode) == "" {
		return NormalizedAttendanceEvent{}, errors.New("external attendance pull: employee_code is required in event data")
	}
	punch := 0
	switch strings.ToLower(strings.TrimSpace(event.Type)) {
	case "out", "punch_out", "checkout":
		punch = 1
	}
	return NormalizedAttendanceEvent{ExternalEventID: strings.TrimSpace(event.ExternalEventID), EmployeeCode: strings.TrimSpace(data.EmployeeCode), Data: string(event.Data), OccurredAt: occurred.UTC(), Punch: punch}, nil
}
