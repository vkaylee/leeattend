package service

import (
	"context"
	"errors"
	"testing"
	"time"
)

type officeWorkerStore struct {
	failed bool
}

func (s *officeWorkerStore) ClaimScheduleOfficeJob(context.Context, time.Duration) (string, []byte, int, string, bool, error) {
	return "job", []byte(`{"approval_instance_id":"a","employer_id":"b","employment_id":"c","business_date":"2026-01-01","mode":"office"}`), 5, "token", true, nil
}
func (s *officeWorkerStore) HeartbeatScheduleOfficeJob(context.Context, string, string, time.Duration) error {
	return nil
}
func (s *officeWorkerStore) CompleteScheduleOfficeJob(context.Context, string, string, string) error {
	return nil
}
func (s *officeWorkerStore) FailScheduleOfficeJob(context.Context, string, string, int, error) error {
	s.failed = true
	return nil
}

type unavailableOfficePublisher struct{}

func (unavailableOfficePublisher) PublishScheduleOffice(context.Context, []byte, string) (string, error) {
	return "", errors.New("unavailable")
}

func TestScheduleOfficeWorkerFailureUsesOutboxFailurePath(t *testing.T) {
	store := &officeWorkerStore{}
	worker, err := NewScheduleOfficeWorker(store, stubOfficeProducer{}, unavailableOfficePublisher{})
	if err != nil {
		t.Fatal(err)
	}
	worker.process(context.Background(), "job", []byte(`{"approval_instance_id":"a","employer_id":"b","employment_id":"c","business_date":"2026-01-01","mode":"office"}`), 5, "token")
	if !store.failed {
		t.Fatal("expected failed job update")
	}
}

type stubOfficeProducer struct{}

func (stubOfficeProducer) Scan(context.Context) (int, error) { return 0, nil }

type lifecycleOfficeProducer struct {
	scanned chan struct{}
}

func (p lifecycleOfficeProducer) Scan(context.Context) (int, error) {
	select {
	case p.scanned <- struct{}{}:
	default:
	}
	return 0, nil
}

type idleOfficeWorkerStore struct{}

func (idleOfficeWorkerStore) ClaimScheduleOfficeJob(context.Context, time.Duration) (string, []byte, int, string, bool, error) {
	return "", nil, 0, "", false, nil
}
func (idleOfficeWorkerStore) HeartbeatScheduleOfficeJob(context.Context, string, string, time.Duration) error {
	return nil
}
func (idleOfficeWorkerStore) CompleteScheduleOfficeJob(context.Context, string, string, string) error {
	return nil
}
func (idleOfficeWorkerStore) FailScheduleOfficeJob(context.Context, string, string, int, error) error {
	return nil
}

func TestScheduleOfficeWorkerStartsInitialScanAndShutsDown(t *testing.T) {
	scanned := make(chan struct{}, 1)
	worker, err := NewScheduleOfficeWorker(idleOfficeWorkerStore{}, lifecycleOfficeProducer{scanned: scanned}, unavailableOfficePublisher{})
	if err != nil {
		t.Fatal(err)
	}
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	worker.Start(ctx)
	select {
	case <-scanned:
	case <-time.After(time.Second):
		t.Fatal("worker did not perform initial scan")
	}
	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), time.Second)
	defer shutdownCancel()
	if err := worker.Shutdown(shutdownCtx); err != nil {
		t.Fatalf("shutdown worker: %v", err)
	}
}
