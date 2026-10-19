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

// CreateResetToken stores a single-use password reset token mapped to the user ID.
func (c *Client) CreateResetToken(ctx context.Context, token string, userID uuid.UUID, ttl time.Duration) error {
	client, err := c.Shared()
	if err != nil {
		return err
	}
	if client == nil {
		return ErrNilRedisClient
	}
	return client.Set(ctx, KeyPasswordReset(token), userID.String(), ttl).Err()
}

// ConsumeResetToken atomically reads and deletes a reset token.
func (c *Client) ConsumeResetToken(ctx context.Context, token string) (uuid.UUID, error) {
	client, err := c.Shared()
	if err != nil {
		return uuid.Nil, err
	}
	if client == nil {
		return uuid.Nil, ErrNilRedisClient
	}
	script := `local v = redis.call('GET', KEYS[1])
if not v then return nil end
redis.call('DEL', KEYS[1])
return v`
	value, err := client.Eval(ctx, script, []string{KeyPasswordReset(token)}).Text()
	if err != nil {
		return uuid.Nil, err
	}
	return uuid.Parse(value)
}

// SetUserRevocation records a user-wide JWT cutoff timestamp for revocation checks.
func (c *Client) SetUserRevocation(ctx context.Context, userID uuid.UUID, ttl time.Duration) error {
	client, err := c.Shared()
	if err != nil {
		return err
	}
	if client == nil {
		return ErrNilRedisClient
	}
	now := strconv.FormatInt(time.Now().UTC().Unix(), 10)
	return client.Set(ctx, KeyUserRevokedAt(userID), now, ttl).Err()
}
