package repository

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// AuthRepository implements session and user persistence against the system database.
type AuthRepository struct{ DB DBTX }

var userColumns = `id, email, password_hash, full_name, avatar_url, bio, preferred_language, created_at, avatar_file_id`

func (r AuthRepository) FindUserByEmail(ctx context.Context, email string) (service.UserRecord, error) {
	return r.findUser(ctx, `SELECT `+userColumns+` FROM users WHERE lower(email) = lower($1) LIMIT 1`, email)
}
func (r AuthRepository) FindUserByID(ctx context.Context, id uuid.UUID) (service.UserRecord, error) {
	return r.findUser(ctx, `SELECT `+userColumns+` FROM users WHERE id = $1 LIMIT 1`, id)
}
func (r AuthRepository) findUser(ctx context.Context, query string, arg any) (service.UserRecord, error) {
	db, err := requireDB(r.DB, "auth repository")
	if err != nil {
		return service.UserRecord{}, err
	}
	var user service.UserRecord
	var createdAt *time.Time
	if err := db.QueryRow(ctx, query, arg).Scan(&user.ID, &user.Email, &user.PasswordHash, &user.FullName, &user.AvatarURL, &user.Bio, &user.PreferredLanguage, &createdAt, &user.AvatarFileID); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return service.UserRecord{}, service.ErrInvalidCredentials
		}
		return service.UserRecord{}, fmt.Errorf("find auth user: %w", err)
	}
	if createdAt == nil {
		user.CreatedAt = time.Now().UTC()
	} else {
		user.CreatedAt = createdAt.UTC()
	}
	return user, nil
}

func (r AuthRepository) UpdateUserProfile(ctx context.Context, userID uuid.UUID, req service.UpdateUserProfileRequest) (service.UserRecord, error) {
	db, err := requireDB(r.DB, "auth repository")
	if err != nil {
		return service.UserRecord{}, err
	}
	sets := make([]string, 0, 4)
	args := make([]any, 0, 5)
	add := func(column string, value *string) {
		if value == nil {
			return
		}
		args = append(args, *value)
		sets = append(sets, fmt.Sprintf("%s = $%d", column, len(args)))
	}
	add("full_name", req.FullName)
	add("avatar_url", req.AvatarURL)
	add("bio", req.Bio)
	add("preferred_language", req.PreferredLanguage)
	args = append(args, userID)
	if len(sets) > 0 {
		query := fmt.Sprintf("UPDATE users SET %s WHERE id = $%d", strings.Join(sets, ", "), len(args))
		if _, err := db.Exec(ctx, query, args...); err != nil {
			return service.UserRecord{}, fmt.Errorf("update user profile: %w", err)
		}
	}
	user, err := r.FindUserByID(ctx, userID)
	if err != nil {
		return service.UserRecord{}, err
	}
	return user, nil
}

func (r AuthRepository) ListOrganizationUsers(ctx context.Context) ([]service.UserRecord, error) {
	db, err := requireDB(r.DB, "auth repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT u.id, u.email, u.password_hash, u.full_name, u.avatar_url, u.bio, u.preferred_language, u.created_at, u.avatar_file_id
FROM users u
INNER JOIN user_roles ur ON ur.user_id = u.id
ORDER BY u.created_at DESC`)
	if err != nil {
		return nil, fmt.Errorf("query organization users: %w", err)
	}
	defer rows.Close()
	users := []service.UserRecord{}
	for rows.Next() {
		var user service.UserRecord
		var createdAt *time.Time
		if err := rows.Scan(&user.ID, &user.Email, &user.PasswordHash, &user.FullName, &user.AvatarURL, &user.Bio, &user.PreferredLanguage, &createdAt, &user.AvatarFileID); err != nil {
			return nil, fmt.Errorf("scan organization user: %w", err)
		}
		if createdAt == nil {
			user.CreatedAt = time.Now().UTC()
		} else {
			user.CreatedAt = createdAt.UTC()
		}
		users = append(users, user)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate organization users: %w", err)
	}
	return users, nil
}
func (r AuthRepository) CreateSession(ctx context.Context, session service.SessionRecord) error {
	db, err := requireDB(r.DB, "auth repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO sessions (id, user_id, token, expires_at) VALUES ($1, $2, $3, $4)`, session.ID, session.UserID, session.TokenHash, session.ExpiresAt)
	if err != nil {
		return fmt.Errorf("create auth session: %w", err)
	}
	return nil
}
func (r AuthRepository) FindSessionByTokenHash(ctx context.Context, tokenHash string) (service.SessionRecord, error) {
	db, err := requireDB(r.DB, "auth repository")
	if err != nil {
		return service.SessionRecord{}, err
	}
	var session service.SessionRecord
	if err := db.QueryRow(ctx, `SELECT id, user_id, token, expires_at, revoked_at FROM sessions WHERE token = $1 LIMIT 1`, tokenHash).Scan(&session.ID, &session.UserID, &session.TokenHash, &session.ExpiresAt, &session.RevokedAt); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return service.SessionRecord{}, service.ErrSessionNotFound
		}
		return service.SessionRecord{}, fmt.Errorf("find auth session: %w", err)
	}
	return session, nil
}
func (r AuthRepository) RevokeSession(ctx context.Context, id uuid.UUID, revokedAt time.Time) error {
	db, err := requireDB(r.DB, "auth repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE sessions SET revoked_at = $2 WHERE id = $1 AND revoked_at IS NULL`, id, revokedAt)
	if err != nil {
		return fmt.Errorf("revoke auth session: %w", err)
	}
	return nil
}
func (r AuthRepository) RevokeAllSessions(ctx context.Context, userID uuid.UUID, revokedAt time.Time) error {
	db, err := requireDB(r.DB, "auth repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE sessions SET revoked_at = $2 WHERE user_id = $1 AND revoked_at IS NULL`, userID, revokedAt)
	if err != nil {
		return fmt.Errorf("revoke all auth sessions: %w", err)
	}
	return nil
}

func (r AuthRepository) CreateUserWithAdminCheck(ctx context.Context, user service.UserRecord) (bool, error) {
	db, err := requireDB(r.DB, "auth repository")
	if err != nil {
		return false, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return false, errors.New("auth repository: database does not support transactions")
	}
	tx, err := beginner.Begin(ctx)
	if err != nil {
		return false, fmt.Errorf("begin register transaction: %w", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	if _, err := tx.Exec(ctx, `SELECT pg_advisory_xact_lock(1337)`); err != nil {
		return false, fmt.Errorf("acquire registration lock: %w", err)
	}
	var count int64
	if err := tx.QueryRow(ctx, `SELECT count(*) FROM users`).Scan(&count); err != nil {
		return false, fmt.Errorf("count users: %w", err)
	}
	isFirst := count == 0
	if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash, created_at, full_name) VALUES ($1, $2, $3, NOW(), $4)`, user.ID, user.Email, user.PasswordHash, user.FullName); err != nil {
		if isUniqueViolation(err) {
			return false, service.ErrEmailExists
		}
		return false, fmt.Errorf("insert user: %w", err)
	}
	if err := tx.Commit(ctx); err != nil {
		return false, fmt.Errorf("commit register transaction: %w", err)
	}
	return isFirst, nil
}

func (r AuthRepository) UpdatePasswordHash(ctx context.Context, userID uuid.UUID, hash string) error {
	db, err := requireDB(r.DB, "auth repository")
	if err != nil {
		return err
	}
	if _, err := db.Exec(ctx, `UPDATE users SET password_hash = $2 WHERE id = $1`, userID, hash); err != nil {
		return fmt.Errorf("update password hash: %w", err)
	}
	return nil
}
