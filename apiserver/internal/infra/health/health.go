package health

import (
	"context"
	"encoding/json"
	"net/http"
	"sync/atomic"
	"time"

	"github.com/vkaylee/leeattend/apiserver/internal/db/pool"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/redis"
)

type Status string

const (
	StatusHealthy   Status = "healthy"
	StatusUnhealthy Status = "unhealthy"
)

type Response struct {
	Status        Status            `json:"status"`
	Version       string            `json:"version"`
	UptimeSeconds int64             `json:"uptime_seconds"`
	Checks        map[string]string `json:"checks,omitempty"`
}

type Handler struct {
	version   string
	startTime time.Time
	poolMgr   *pool.Manager
	redisCli  *redis.Client
	isLive    atomic.Bool
}

func NewHandler(version string, poolMgr *pool.Manager, redisCli *redis.Client) *Handler {
	h := &Handler{version: version, startTime: time.Now().UTC(), poolMgr: poolMgr, redisCli: redisCli}
	h.isLive.Store(true)
	return h
}

func (h *Handler) SetLive(live bool) { h.isLive.Store(live) }

// Liveness succeeds without DB or Redis dependencies.
func (h *Handler) Liveness(w http.ResponseWriter, _ *http.Request) {
	status := StatusHealthy
	code := http.StatusOK
	if !h.isLive.Load() {
		status = StatusUnhealthy
		code = http.StatusServiceUnavailable
	}
	writeJSON(w, code, Response{Status: status, Version: h.version, UptimeSeconds: h.uptime()})
}

// Readiness performs database and Redis dependency checks.
func (h *Handler) Readiness(w http.ResponseWriter, r *http.Request) {
	ctx, cancel := context.WithTimeout(r.Context(), 3*time.Second)
	defer cancel()

	checks := make(map[string]string, 2)
	healthy := true
	if h.poolMgr == nil {
		checks["database"] = "ok"
	} else {
		sysPool, err := h.poolMgr.SystemPool()
		if err != nil || sysPool == nil {
			checks["database"] = "error"
			healthy = false
		} else {
			var one int
			if err := sysPool.QueryRow(ctx, "SELECT 1").Scan(&one); err != nil || one != 1 {
				checks["database"] = "error"
				healthy = false
			} else {
				checks["database"] = "ok"
			}
		}
	}
	if h.redisCli == nil {
		checks["redis"] = "disabled"
	} else if err := h.redisCli.Ping(ctx); err != nil {
		checks["redis"] = "error"
		healthy = false
	} else {
		checks["redis"] = "ok"
	}

	code := http.StatusOK
	status := StatusHealthy
	if !healthy {
		code = http.StatusInternalServerError
		status = StatusUnhealthy
	}
	writeJSON(w, code, Response{Status: status, Version: h.version, UptimeSeconds: h.uptime(), Checks: checks})
}

func (h *Handler) uptime() int64 {
	seconds := int64(time.Since(h.startTime).Seconds())
	if seconds < 0 {
		return 0
	}
	return seconds
}

func writeJSON(w http.ResponseWriter, status int, value Response) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(value)
}
