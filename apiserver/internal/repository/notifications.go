package repository

import (
	"context"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type NotificationRepository struct {
	SystemDB DBTX
}

func (r NotificationRepository) List(ctx context.Context, userID uuid.UUID, limit int, cursor *time.Time) (api.NotificationResponse, error) {
	db, err := requireDB(r.SystemDB, "notification repository")
	if err != nil {
		return api.NotificationResponse{}, err
	}
	rows, err := db.Query(ctx, `SELECT id, user_id, type, title, content, is_read, link, created_at FROM notifications WHERE user_id = $1 AND ($2::timestamptz IS NULL OR created_at < $2) ORDER BY created_at DESC LIMIT $3`, userID, cursor, limit+1)
	if err != nil {
		return api.NotificationResponse{}, err
	}
	defer rows.Close()
	items := make([]api.NotificationRecord, 0, limit)
	for rows.Next() {
		var item api.NotificationRecord
		var createdAt time.Time
		if err := rows.Scan(&item.ID, &item.UserID, &item.Type, &item.Title, &item.Content, &item.IsRead, &item.Link, &createdAt); err != nil {
			return api.NotificationResponse{}, err
		}
		item.CreatedAt = createdAt.UTC().Format(time.RFC3339Nano)
		items = append(items, item)
	}
	if err := rows.Err(); err != nil {
		return api.NotificationResponse{}, err
	}
	var next *string
	if len(items) > limit {
		value := items[limit-1].CreatedAt
		next = &value
		items = items[:limit]
	}
	var unread int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM notifications WHERE user_id = $1 AND is_read = false`, userID).Scan(&unread); err != nil {
		return api.NotificationResponse{}, err
	}
	return api.NotificationResponse{Notifications: items, NextCursor: next, UnreadCount: unread}, nil
}
func (r NotificationRepository) MarkRead(ctx context.Context, userID, notificationID uuid.UUID) error {
	db, err := requireDB(r.SystemDB, "notification repository")
	if err != nil {
		return err
	}
	cmd, err := db.Exec(ctx, `UPDATE notifications SET is_read = true WHERE id = $1 AND user_id = $2`, notificationID, userID)
	if err != nil {
		return err
	}
	if cmd.RowsAffected() == 0 {
		return api.ErrNotificationNotFound
	}
	return nil
}
func (r NotificationRepository) MarkAllRead(ctx context.Context, userID uuid.UUID) error {
	db, err := requireDB(r.SystemDB, "notification repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE notifications SET is_read = true WHERE user_id = $1 AND is_read = false`, userID)
	return err
}

var _ = pgx.ErrNoRows
