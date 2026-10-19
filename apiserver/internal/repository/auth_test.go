package repository

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type fakeRow struct {
	scanFn func(dest ...any) error
}

func (f fakeRow) Scan(dest ...any) error {
	if f.scanFn != nil {
		return f.scanFn(dest...)
	}
	return pgx.ErrNoRows
}

type fakeDBTX struct {
	queryRowFn func(ctx context.Context, sql string, args ...any) pgx.Row
	execFn     func(ctx context.Context, sql string, arguments ...any) (pgconn.CommandTag, error)
	queryFn    func(ctx context.Context, sql string, args ...any) (pgx.Rows, error)
}

func (f *fakeDBTX) QueryRow(ctx context.Context, sql string, args ...any) pgx.Row {
	if f.queryRowFn != nil {
		return f.queryRowFn(ctx, sql, args...)
	}
	return fakeRow{}
}

func (f *fakeDBTX) Exec(ctx context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
	if f.execFn != nil {
		return f.execFn(ctx, sql, arguments...)
	}
	return pgconn.NewCommandTag("INSERT 0 1"), nil
}

func (f *fakeDBTX) Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error) {
	if f.queryFn != nil {
		return f.queryFn(ctx, sql, args...)
	}
	return nil, nil
}

func TestAuthRepository_FindUserByEmail(t *testing.T) {
	expectedID := uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(ctx context.Context, sql string, args ...any) pgx.Row {
			return fakeRow{
				scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = expectedID
					*(dest[1].(*string)) = "user@example.com"
					*(dest[2].(*string)) = "hash"
					*(dest[3].(*string)) = "Full Name"
					return nil
				},
			}
		},
	}

	repo := AuthRepository{DB: db}
	user, err := repo.FindUserByEmail(context.Background(), "user@example.com")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if user.ID != expectedID || user.Email != "user@example.com" {
		t.Errorf("unexpected user record: %+v", user)
	}
}

func TestAuthRepository_FindSessionByTokenHash(t *testing.T) {
	expectedID := uuid.New()
	expectedUser := uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(ctx context.Context, sql string, args ...any) pgx.Row {
			return fakeRow{
				scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = expectedID
					*(dest[1].(*uuid.UUID)) = expectedUser
					*(dest[2].(*string)) = "hashed-token"
					*(dest[3].(*time.Time)) = time.Now().Add(time.Hour)
					*(dest[4].(**time.Time)) = nil
					return nil
				},
			}
		},
	}

	repo := AuthRepository{DB: db}
	sess, err := repo.FindSessionByTokenHash(context.Background(), "hashed-token")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if sess.ID != expectedID || sess.UserID != expectedUser {
		t.Errorf("unexpected session record: %+v", sess)
	}
}

func TestAuthRepository_CreateAndRevokeSession(t *testing.T) {
	execCount := 0
	db := &fakeDBTX{
		execFn: func(ctx context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
			execCount++
			return pgconn.NewCommandTag("UPDATE 1"), nil
		},
	}

	repo := AuthRepository{DB: db}
	if err := repo.CreateSession(context.Background(), service.SessionRecord{
		ID:        uuid.New(),
		UserID:    uuid.New(),
		TokenHash: "abc",
		ExpiresAt: time.Now(),
	}); err != nil {
		t.Fatalf("create session error: %v", err)
	}

	if err := repo.RevokeSession(context.Background(), uuid.New(), time.Now()); err != nil {
		t.Fatalf("revoke session error: %v", err)
	}

	if err := repo.RevokeAllSessions(context.Background(), uuid.New(), time.Now()); err != nil {
		t.Fatalf("revoke all sessions error: %v", err)
	}

	if execCount != 3 {
		t.Errorf("expected 3 exec calls, got %d", execCount)
	}
}
