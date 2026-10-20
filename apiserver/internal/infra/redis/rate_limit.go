package redis

import (
	"context"

	goredis "github.com/redis/go-redis/v9"
)

var rateLimitLuaScript = goredis.NewScript(`
local current = redis.call("INCR", KEYS[1])
if current == 1 then
    redis.call("EXPIRE", KEYS[1], ARGV[1])
end
return current
`)

// CheckRateLimit increments the key and sets TTL if it is the first hit, returning the new count.
func (c *Client) CheckRateLimit(ctx context.Context, key string, windowSecs uint64) (uint64, error) {
	client, err := c.Shared()
	if err != nil || client == nil {
		return 0, ErrNilRedisClient
	}
	res, err := rateLimitLuaScript.Run(ctx, client, []string{key}, windowSecs).Int64()
	if err != nil {
		return 0, err
	}
	return uint64(res), nil
}
