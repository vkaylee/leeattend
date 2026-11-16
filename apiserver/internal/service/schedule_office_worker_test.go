package service

import (
	"context"
	"errors"
	"testing"
	"time"
)

type officeWorkerStore struct {
	failed        bool
	completed     bool
	heartbeatErr  error
	failureReason error
}

func (s *officeWorkerStore) ClaimScheduleOfficeJob(context.Context, time.Duration) (string, []byte, int, string, bool, error) {
	return "job", []byte(`{"approval_instance_id":"a","employer_id":"b","employment_id":"c","business_date":"2026-01-01","mode":"office"}`), 5, "token", true, nil
}
func (s *officeWorkerStore) HeartbeatScheduleOfficeJob(context.Context, string, string, time.Duration) error {
	return s.heartbeatErr
}
func (s *officeWorkerStore) CompleteScheduleOfficeJob(context.Context, string, string, string) error {
	s.completed = true
	return nil
}
func (s *officeWorkerStore) FailScheduleOfficeJob(_ context.Context, _ string, _ string, _ int, cause error) error {
	s.failed = true
	s.failureReason = cause
	return nil
}

type unavailableOfficePublisher struct{}

func (unavailableOfficePublisher) PublishScheduleOffice(context.Context, []byte, string) (string, error) {
	return "", errors.New("unavailable")
}

type successfulOfficePublisher struct {
	started chan struct{}
	release chan struct{}
}

func (p successfulOfficePublisher) PublishScheduleOffice(context.Context, []byte, string) (string, error) {
	close(p.started)
	<-p.release
	return "00000000-0000-0000-0000-000000000001", nil
}

func TestScheduleOfficeWorkerLeaseLossPreventsCompletion(t *testing.T) {
	store := &officeWorkerStore{heartbeatErr: errors.New("lease lost")}
	publisher := successfulOfficePublisher{started: make(chan struct{}), release: make(chan struct{})}
	worker, err := NewScheduleOfficeWorker(store, stubOfficeProducer{}, publisher)
	if err != nil {
		t.Fatal(err)
	}
	worker.heartbeatInterval = time.Millisecond
	done := make(chan struct{})
	go func() {
		worker.process(context.Background(), "job", []byte(`{"approval_instance_id":"a","employer_id":"b","employment_id":"c","business_date":"2026-01-01","mode":"office"}`), 1, "token")
		close(done)
	}()
	select {
	case <-publisher.started:
	case <-time.After(time.Second):
		t.Fatal("publisher did not start")
	}
	time.Sleep(5 * time.Millisecond)
	close(publisher.release)
	select {
	case <-done:
	case <-time.After(time.Second):
		t.Fatal("worker process did not finish")
	}
	if store.completed {
		t.Fatal("lease loss must prevent completion")
	}
	if !store.failed {
		t.Fatal("lease loss must enter failure path")
	}
}

func TestScheduleOfficeWorkerMalformedPayloadFailsWithoutPublishing(t *testing.T) {
	store := &officeWorkerStore{}
	worker, err := NewScheduleOfficeWorker(store, stubOfficeProducer{}, successfulOfficePublisher{started: make(chan struct{}), release: make(chan struct{})})
	if err != nil {
		t.Fatal(err)
	}
	worker.process(context.Background(), "job", []byte(`{"task":"poison"}`), 1, "token")
	if !store.failed || store.failureReason == nil || !errors.Is(store.failureReason, ErrScheduleOfficeMalformedPayload) {
		t.Fatalf("malformed payload failure=%v, want malformed payload", store.failureReason)
	}
	if store.completed {
		t.Fatal("malformed payload must not complete")
	}
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
