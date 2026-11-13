package service

import (
	"context"
	"encoding/json"
	"errors"
	"sync"
	"time"
)

const (
	ScheduleOfficeScanInterval = 15 * time.Minute
	ScheduleOfficeLease        = 5 * time.Minute
	ScheduleOfficeHeartbeat    = time.Minute
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
	store     ScheduleOfficeJobStore
	producer  ScheduleOfficeProducer
	publisher ScheduleOfficePublisher
	done      chan struct{}
	closed    chan struct{}
	once      sync.Once
	closeOnce sync.Once
}

func NewScheduleOfficeWorker(store ScheduleOfficeJobStore, producer ScheduleOfficeProducer, publisher ScheduleOfficePublisher) (*ScheduleOfficeWorker, error) {
	if store == nil || producer == nil {
		return nil, ErrScheduleOfficeWorkerStore
	}
	return &ScheduleOfficeWorker{store: store, producer: producer, publisher: publisher, done: make(chan struct{}), closed: make(chan struct{})}, nil
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
	w.scanAndDrain(ctx)
	ticker := time.NewTicker(ScheduleOfficeScanInterval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-w.done:
			return
		case <-ticker.C:
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
	if w.publisher == nil {
		_ = w.store.FailScheduleOfficeJob(ctx, id, lease, attempt, ErrScheduleOfficePublisher)
		return
	}
	workCtx, cancel := context.WithCancel(ctx)
	heartbeatDone := make(chan struct{})
	go w.heartbeat(workCtx, id, lease, heartbeatDone)
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
	_ = w.store.CompleteScheduleOfficeJob(ctx, id, lease, batchID)
}

func (w *ScheduleOfficeWorker) heartbeat(ctx context.Context, id, lease string, done chan<- struct{}) {
	defer close(done)
	ticker := time.NewTicker(ScheduleOfficeHeartbeat)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if err := w.store.HeartbeatScheduleOfficeJob(ctx, id, lease, ScheduleOfficeLease); err != nil {
				return
			}
		}
	}
}
