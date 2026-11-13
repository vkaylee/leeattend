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
