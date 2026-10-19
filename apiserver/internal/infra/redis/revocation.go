package redis

import (
	"context"
	"strconv"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
)

// IsSignatureRevoked reads JWT signature blocklist. Redis outages are fail-open by contract.
func (c *Client) IsSignatureRevoked(ctx context.Context, signature string) (bool, error) {
	client, err := c.Shared()
	if err != nil || client == nil {
		return false, err
	}
	value, err := client.Get(ctx, KeyJWTRevoked(signature)).Result()
	if err != nil {
		return false, err
	}
	return value != "", nil
}

// UserRevokedAt reads global user revocation timestamp. Redis outages are fail-open for access checks.
func (c *Client) UserRevokedAt(ctx context.Context, userID uuid.UUID) (time.Time, error) {
	client, err := c.Shared()
	if err != nil || client == nil {
		return time.Time{}, err
	}
	value, err := client.Get(ctx, KeyUserRevokedAt(userID)).Result()
	if err != nil {
		return time.Time{}, err
	}
	seconds, err := strconv.ParseInt(value, 10, 64)
	if err != nil {
		return time.Time{}, err
	}
	return time.Unix(seconds, 0).UTC(), nil
}

// RevocationKeys keeps key construction in one package while ensuring auth and Redis contracts match.
func RevocationKeys(token string, userID uuid.UUID) (string, string) {
	return auth.RevokedSignatureKey(token), KeyUserRevokedAt(userID)
}
