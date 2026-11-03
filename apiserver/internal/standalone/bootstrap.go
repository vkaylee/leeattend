package standalone

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
)

var ErrInvalidBootstrap = errors.New("standalone: invalid bootstrap configuration")

// Bootstrap creates the singleton organization and its first administrator.
// It is idempotent for an already-complete installation and refuses conflicting
// existing state, preventing accidental takeover of a non-standalone database.
func Bootstrap(ctx context.Context, db interface {
	Begin(context.Context) (pgx.Tx, error)
}, workspaceID uuid.UUID, workspaceName, email, password string) error {
	if db == nil || workspaceID == uuid.Nil || strings.TrimSpace(workspaceName) == "" || strings.TrimSpace(email) == "" || strings.TrimSpace(password) == "" {
		return ErrInvalidBootstrap
	}
	passwordHash, err := auth.HashPassword(password)
	if err != nil {
		return fmt.Errorf("standalone: hash bootstrap password: %w", err)
	}
	tx, err := db.Begin(ctx)
	if err != nil {
		return fmt.Errorf("standalone: begin bootstrap: %w", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	if err := EnsureDefaultRolesAndPermissions(ctx, tx); err != nil {
		return err
	}

	var workspaceCount int
	if err := tx.QueryRow(ctx, `SELECT count(*) FROM workspaces`).Scan(&workspaceCount); err != nil {
		return fmt.Errorf("standalone: count organizations: %w", err)
	}
	if workspaceCount > 1 {
		return fmt.Errorf("%w: multiple organizations exist", ErrInvalidBootstrap)
	}

	var existingWorkspace uuid.UUID
	if workspaceCount == 1 {
		if err := tx.QueryRow(ctx, `SELECT id FROM workspaces LIMIT 1`).Scan(&existingWorkspace); err != nil {
			return fmt.Errorf("standalone: read organization: %w", err)
		}
		if existingWorkspace != workspaceID {
			return fmt.Errorf("%w: configured organization does not match database", ErrInvalidBootstrap)
		}
	} else if _, err := tx.Exec(ctx, `INSERT INTO workspaces (id, name) VALUES ($1, $2)`, workspaceID, strings.TrimSpace(workspaceName)); err != nil {
		return fmt.Errorf("standalone: create organization: %w", err)
	}

	var userID uuid.UUID
	err = tx.QueryRow(ctx, `SELECT id FROM users WHERE lower(email) = lower($1) LIMIT 1`, strings.TrimSpace(email)).Scan(&userID)
	if errors.Is(err, pgx.ErrNoRows) {
		userID = uuid.New()
		if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash, created_at, full_name) VALUES ($1, $2, $3, NOW(), $4)`, userID, strings.ToLower(strings.TrimSpace(email)), passwordHash, "Standalone Administrator"); err != nil {
			return fmt.Errorf("standalone: create administrator: %w", err)
		}
	} else if err != nil {
		return fmt.Errorf("standalone: read administrator: %w", err)
	}

	var roleID uuid.UUID
	if err := tx.QueryRow(ctx, `SELECT id FROM workspace_roles WHERE workspace_id IS NULL AND name = 'owner' LIMIT 1`).Scan(&roleID); err != nil {
		return fmt.Errorf("standalone: read owner role: %w", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING`, workspaceID, userID, roleID); err != nil {
		return fmt.Errorf("standalone: assign owner role: %w", err)
	}
	if err := tx.Commit(ctx); err != nil {
		return fmt.Errorf("standalone: commit bootstrap: %w", err)
	}
	return nil
}
