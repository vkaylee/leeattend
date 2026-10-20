package redis

import (
	"context"
	"errors"
	"strconv"
	"time"

	goredis "github.com/redis/go-redis/v9"
)

// IsLocked checks the per-email lockout key. Redis failure is fail-open for login availability.
func (c *Client) IsLocked(ctx context.Context, email string) (bool, error) {
	client, err := c.Shared()
	if err != nil || client == nil {
		return false, nil
	}
	value, err := client.Get(ctx, KeyLoginLockout(email)).Result()
	if errors.Is(err, goredis.Nil) {
		return false, nil
	}
	if err != nil {
		return false, nil
	}
	return value != "", nil
}

// RecordFailure increments attempts and creates exponential lockout from attempt five.
func (c *Client) RecordFailure(ctx context.Context, email string) error {
	client, err := c.Shared()
	if err != nil || client == nil {
		return nil
	}
	attempts, err := client.Incr(ctx, KeyLoginAttempts(email)).Result()
	if err != nil {
		return nil
	}
	if attempts == 1 {
		_ = client.Expire(ctx, KeyLoginAttempts(email), TTLLoginAttempts).Err()
	}
	if attempts < 5 {
		return nil
	}
	exponent := attempts - 5
	if exponent > 6 {
		exponent = 6
	}
	lockSeconds := int64(TTLLoginLockoutBase.Seconds()) << exponent
	return client.SetEx(ctx, KeyLoginLockout(email), strconv.FormatInt(attempts, 10), secondsDuration(lockSeconds)).Err()
}

func (c *Client) ClearFailures(ctx context.Context, email string) error {
	client, err := c.Shared()
	if err != nil || client == nil {
		return nil
	}
	return client.Del(ctx, KeyLoginAttempts(email), KeyLoginLockout(email)).Err()
}

func secondsDuration(seconds int64) time.Duration { return time.Duration(seconds) * time.Second }
