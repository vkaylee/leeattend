package repository

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func assertMonitorDomainErrors(t *testing.T, repo MonitorRepository, want error) {
	t.Helper()
	ctx, id := context.Background(), uuid.New()
	_, _, alertErr := repo.GetAlert(ctx, id)
	_, mutationErr := repo.MutateAlert(ctx, id, "acknowledge", "reason", id)
	_, jobErr := repo.GetDurableJob(ctx, id)
	for name, err := range map[string]error{"alert": alertErr, "mutation": mutationErr, "job": jobErr} {
		if !errors.Is(err, want) {
			t.Errorf("%s: got %v, want %v", name, err, want)
		}
	}
}

func TestMonitorMissingResourcesNilSystemDB(t *testing.T) {
	assertMonitorDomainErrors(t, MonitorRepository{}, api.ErrMonitorNotFound)
}

func TestMonitorMissingResourcesNoRows(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(context.Context, string, ...any) pgx.Row {
			return fakeRow{scanFn: func(...any) error { return pgx.ErrNoRows }}
		},
	}
	assertMonitorDomainErrors(t, MonitorRepository{SystemDB: db}, api.ErrMonitorNotFound)
}

func TestMonitorUnexpectedDBErrorPropagates(t *testing.T) {
	dbErr := errors.New("connection reset by peer")
	db := &fakeDBTX{
		queryRowFn: func(context.Context, string, ...any) pgx.Row {
			return fakeRow{scanFn: func(...any) error { return dbErr }}
		},
	}
	assertMonitorDomainErrors(t, MonitorRepository{SystemDB: db}, dbErr)
}

func TestMonitorMutateAlertConflictReturnsDomainSentinel(t *testing.T) {
	ctx, id := context.Background(), uuid.New()
	now := time.Now().UTC()
	makeAlertRow := func(status string) fakeRow {
		return fakeRow{scanFn: func(dest ...any) error {
			*(dest[0].(*uuid.UUID)) = id
			*(dest[1].(*string)) = "queue_pressure"
			*(dest[2].(*string)) = "critical"
			*(dest[3].(*string)) = status
			*(dest[4].(*string)) = "title"
			*(dest[5].(*string)) = "summary"
			*(dest[6].(*string)) = "worker"
			*(dest[7].(*time.Time)) = now
			*(dest[8].(*time.Time)) = now
			*(dest[9].(*int32)) = 1
			*(dest[10].(**time.Time)) = nil
			return nil
		}}
	}

	ackCalls := 0
	ackRepo := MonitorRepository{SystemDB: &fakeDBTX{
		queryRowFn: func(context.Context, string, ...any) pgx.Row {
			ackCalls++
			if ackCalls == 1 {
				return makeAlertRow("resolved")
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = false
				return nil
			}}
		},
	}}
	if _, err := ackRepo.MutateAlert(ctx, id, "acknowledge", "reason", id); !errors.Is(err, api.ErrMonitorConflict) {
		t.Errorf("acknowledge resolved alert: got %v, want ErrMonitorConflict", err)
	}

	resRepo := MonitorRepository{SystemDB: &fakeDBTX{
		queryRowFn: func(context.Context, string, ...any) pgx.Row {
			return makeAlertRow("suppressed")
		},
	}}
	if _, err := resRepo.MutateAlert(ctx, id, "resolve", "reason", id); !errors.Is(err, api.ErrMonitorConflict) {
		t.Errorf("resolve suppressed alert: got %v, want ErrMonitorConflict", err)
	}
}
