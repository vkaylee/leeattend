package service

import (
	"context"
	"encoding/json"
	"errors"
	"sync"
	"time"

	"github.com/google/uuid"
)

const (
	ScheduleOfficeScanInterval = 15 * time.Minute
	ScheduleOfficeLease        = 5 * time.Minute
	ScheduleOfficeHeartbeat    = time.Minute
	ScheduleOfficeJobType      = "ScheduleOfficeCoverageRefresh"
)

var (
	ErrScheduleOfficeWorkerStore      = errors.New("schedule office worker store is unavailable")
	ErrScheduleOfficePublisher        = errors.New("schedule office publication service is unavailable")
	ErrScheduleOfficeMalformedPayload = errors.New("schedule office job payload is malformed")
)

type ScheduleOfficeJobStore interface {
	ClaimScheduleOfficeJob(context.Context, time.Duration) (string, []byte, int, string, bool, error)
	HeartbeatScheduleOfficeJob(context.Context, string, string, time.Duration) error
	CompleteScheduleOfficeJob(context.Context, string, string, string) error
	FailScheduleOfficeJob(context.Context, string, string, int, error) error
}

type ScheduleOfficePublisher interface {
	PublishScheduleOffice(context.Context, []byte, string) (string, error)
}

type ScheduleOfficeProducer interface {
	Scan(context.Context) (int, error)
}

type ScheduleOfficeWorker struct {
	store             ScheduleOfficeJobStore
	producer          ScheduleOfficeProducer
	publisher         ScheduleOfficePublisher
	done              chan struct{}
	closed            chan struct{}
	once              sync.Once
	closeOnce         sync.Once
	workerID          uuid.UUID
	heartbeatWriter   WorkerHeartbeatWriter
	heartbeatInterval time.Duration
}

type WorkerHeartbeatWriter interface {
	WriteWorkerHeartbeat(context.Context, uuid.UUID, string, *string, time.Time) error
	DeleteWorkerHeartbeat(context.Context, uuid.UUID) error
}

func NewScheduleOfficeWorker(store ScheduleOfficeJobStore, producer ScheduleOfficeProducer, publisher ScheduleOfficePublisher) (*ScheduleOfficeWorker, error) {
	return NewScheduleOfficeWorkerWithHeartbeat(store, producer, publisher, nil)
}

func NewScheduleOfficeWorkerWithHeartbeat(store ScheduleOfficeJobStore, producer ScheduleOfficeProducer, publisher ScheduleOfficePublisher, heartbeatWriter WorkerHeartbeatWriter) (*ScheduleOfficeWorker, error) {
	if store == nil || producer == nil {
		return nil, ErrScheduleOfficeWorkerStore
	}
	return &ScheduleOfficeWorker{
		store: store, producer: producer, publisher: publisher,
		done: make(chan struct{}), closed: make(chan struct{}),
		workerID: uuid.New(), heartbeatWriter: heartbeatWriter, heartbeatInterval: ScheduleOfficeHeartbeat,
	}, nil
}

func (w *ScheduleOfficeWorker) Start(ctx context.Context) {
	if w == nil {
		return
	}
	w.once.Do(func() { go w.run(ctx) })
}

func (w *ScheduleOfficeWorker) Shutdown(ctx context.Context) error {
	if w == nil {
		return nil
	}
	w.closeOnce.Do(func() { close(w.done) })
	select {
	case <-w.closed:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

func (w *ScheduleOfficeWorker) run(ctx context.Context) {
	defer close(w.closed)
	w.publishHeartbeat(ctx, "live", nil)
	defer w.deleteHeartbeat(context.Background())
	w.scanAndDrain(ctx)
	ticker := time.NewTicker(ScheduleOfficeScanInterval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			w.publishHeartbeat(context.Background(), "stopped", nil)
			return
		case <-w.done:
			w.publishHeartbeat(context.Background(), "stopped", nil)
			return
		case <-ticker.C:
			w.publishHeartbeat(ctx, "live", nil)
			w.scanAndDrain(ctx)
		}
	}
}

func (w *ScheduleOfficeWorker) scanAndDrain(ctx context.Context) {
	_, _ = w.producer.Scan(ctx)
	for {
		id, payload, attempt, lease, found, err := w.store.ClaimScheduleOfficeJob(ctx, ScheduleOfficeLease)
		if err != nil || !found {
			return
		}
		w.process(ctx, id, payload, attempt, lease)
	}
}

func (w *ScheduleOfficeWorker) process(ctx context.Context, id string, payload []byte, attempt int, lease string) {
	w.publishHeartbeat(ctx, "busy", stringPtr(ScheduleOfficeJobType))
	defer w.publishHeartbeat(ctx, "live", nil)
	if w.publisher == nil {
		_ = w.store.FailScheduleOfficeJob(ctx, id, lease, attempt, ErrScheduleOfficePublisher)
		return
	}
	workCtx, cancel := context.WithCancel(ctx)
	heartbeatDone := make(chan struct{})
	leaseLost := make(chan error, 1)
	go w.heartbeat(workCtx, id, lease, heartbeatDone, leaseLost)
	defer func() {
		cancel()
		<-heartbeatDone
	}()
	var event struct {
		ApprovalInstanceID string `json:"approval_instance_id"`
		EmployerID         string `json:"employer_id"`
		EmploymentID       string `json:"employment_id"`
		BusinessDate       string `json:"business_date"`
		Mode               string `json:"mode"`
	}
	if err := json.Unmarshal(payload, &event); err != nil || event.ApprovalInstanceID == "" || event.EmployerID == "" || event.EmploymentID == "" || event.BusinessDate == "" || event.Mode != "office" {
		_ = w.store.FailScheduleOfficeJob(ctx, id, lease, attempt, ErrScheduleOfficeMalformedPayload)
		return
	}
	batchID, err := w.publisher.PublishScheduleOffice(ctx, payload, id)
	if err != nil {
		_ = w.store.FailScheduleOfficeJob(ctx, id, lease, attempt, err)
		return
	}
	select {
	case leaseErr := <-leaseLost:
		_ = w.store.FailScheduleOfficeJob(ctx, id, lease, attempt, leaseErr)
		return
	default:
	}
	_ = w.store.CompleteScheduleOfficeJob(ctx, id, lease, batchID)
}

func stringPtr(value string) *string { return &value }

func (w *ScheduleOfficeWorker) publishHeartbeat(ctx context.Context, status string, taskType *string) {
	if w.heartbeatWriter == nil || w.workerID == uuid.Nil {
		return
	}
	_ = w.heartbeatWriter.WriteWorkerHeartbeat(ctx, w.workerID, status, taskType, time.Now().UTC())
}

func (w *ScheduleOfficeWorker) deleteHeartbeat(ctx context.Context) {
	if w.heartbeatWriter == nil || w.workerID == uuid.Nil {
		return
	}
	_ = w.heartbeatWriter.DeleteWorkerHeartbeat(ctx, w.workerID)
}

func (w *ScheduleOfficeWorker) heartbeat(ctx context.Context, id, lease string, done chan<- struct{}, leaseLost chan<- error) {
	defer close(done)
	interval := w.heartbeatInterval
	if interval <= 0 {
		interval = ScheduleOfficeHeartbeat
	}
	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if err := w.store.HeartbeatScheduleOfficeJob(ctx, id, lease, ScheduleOfficeLease); err != nil {
				select {
				case leaseLost <- err:
				default:
				}
				return
			}
		}
	}
}
