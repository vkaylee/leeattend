package repository

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// AuthRepository implements session and user persistence against the system database.
type AuthRepository struct{ DB DBTX }

func (r AuthRepository) FindUserByEmail(ctx context.Context, email string) (service.UserRecord, error) {
	return r.findUser(ctx, `SELECT id, email, password_hash, COALESCE(full_name, '') FROM users WHERE lower(email) = lower($1) LIMIT 1`, email)
}
func (r AuthRepository) FindUserByID(ctx context.Context, id uuid.UUID) (service.UserRecord, error) {
	return r.findUser(ctx, `SELECT id, email, password_hash, COALESCE(full_name, '') FROM users WHERE id = $1 LIMIT 1`, id)
}
func (r AuthRepository) findUser(ctx context.Context, query string, arg any) (service.UserRecord, error) {
	if r.DB == nil {
		return service.UserRecord{}, errors.New("auth repository: nil database")
	}
	var user service.UserRecord
	if err := r.DB.QueryRow(ctx, query, arg).Scan(&user.ID, &user.Email, &user.PasswordHash, &user.FullName); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return service.UserRecord{}, service.ErrInvalidCredentials
		}
		return service.UserRecord{}, fmt.Errorf("find auth user: %w", err)
	}
	return user, nil
}
func (r AuthRepository) CreateSession(ctx context.Context, session service.SessionRecord) error {
	if r.DB == nil {
		return errors.New("auth repository: nil database")
	}
	_, err := r.DB.Exec(ctx, `INSERT INTO sessions (id, user_id, token, expires_at) VALUES ($1, $2, $3, $4)`, session.ID, session.UserID, session.TokenHash, session.ExpiresAt)
	if err != nil {
		return fmt.Errorf("create auth session: %w", err)
	}
	return nil
}
func (r AuthRepository) FindSessionByTokenHash(ctx context.Context, tokenHash string) (service.SessionRecord, error) {
	if r.DB == nil {
		return service.SessionRecord{}, errors.New("auth repository: nil database")
	}
	var session service.SessionRecord
	if err := r.DB.QueryRow(ctx, `SELECT id, user_id, token, expires_at, revoked_at FROM sessions WHERE token = $1 LIMIT 1`, tokenHash).Scan(&session.ID, &session.UserID, &session.TokenHash, &session.ExpiresAt, &session.RevokedAt); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return service.SessionRecord{}, service.ErrSessionNotFound
		}
		return service.SessionRecord{}, fmt.Errorf("find auth session: %w", err)
	}
	return session, nil
}
func (r AuthRepository) RevokeSession(ctx context.Context, id uuid.UUID, revokedAt time.Time) error {
	if r.DB == nil {
		return errors.New("auth repository: nil database")
	}
	_, err := r.DB.Exec(ctx, `UPDATE sessions SET revoked_at = $2 WHERE id = $1 AND revoked_at IS NULL`, id, revokedAt)
	if err != nil {
		return fmt.Errorf("revoke auth session: %w", err)
	}
	return nil
}
func (r AuthRepository) RevokeAllSessions(ctx context.Context, userID uuid.UUID, revokedAt time.Time) error {
	if r.DB == nil {
		return errors.New("auth repository: nil database")
	}
	_, err := r.DB.Exec(ctx, `UPDATE sessions SET revoked_at = $2 WHERE user_id = $1 AND revoked_at IS NULL`, userID, revokedAt)
	if err != nil {
		return fmt.Errorf("revoke all auth sessions: %w", err)
	}
	return nil
}
