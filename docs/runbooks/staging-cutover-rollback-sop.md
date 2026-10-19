# Staging Cutover, Canary Routing & Rollback Standard Operating Procedure (SOP)

**Document ID:** `SOP-P8-01`  
**Phase:** Phase P8 (Staging Deployment, Shadow Traffic, Canary Routing, and Rollback Drills)  
**Ledger Tasks:** P8-01 through P8-09  
**Target Services:** `apiserver` (Rust Baseline, Port 3000) & `api-go` (Go Candidate, Port 3001)  
**Gateway Router:** Caddy Reverse Proxy (Ports 80, 443, Admin 2019)  
**Last Updated:** 2026-10-20  

---

## 1. Architectural Topology & Non-Interference Guarantees

```
                              [Internet / Staging Ingress]
                                           │
                                           ▼
                             [Caddy Edge Gateway :80/:443]
                             (Admin & Canary API :2019)
                                    │             │
                Canary % / Flagged  │             │  Default (100% Baseline)
                                    ▼             ▼
                           [api-go :3001]    [apiserver :3000]
                           (Go Candidate)    (Rust Baseline)
                                    │             │
                                    └──────┬──────┘
                                           │
                        ┌──────────────────┴──────────────────┐
                        ▼                                     ▼
                [PgBouncer :6543/:5452]               [Redis :6379]
                        │                                     │
                        ▼                                     ▼
             [PostgreSQL (System/Tenant)]            (Sessions/Blocklist)
```

### Critical Non-Interference Guarantees
1. **Zero Port Collision:** Rust binds `:3000`; Go binds `:3001`. Both communicate over isolated Unix sockets or host-bridged networks without conflict (`D-10`).
2. **Zero Connection Pool Starvation:** Both runtimes query through PgBouncer transaction poolers (`pgbouncer_tx:6543`, `pgbouncer_tenant_tx:6543`). Total backend PostgreSQL connections remain strictly bounded to $\le 50$.
3. **Zero Split-Brain Job Dequeuing:** During canary and shadow phases, the Go background worker is pinned to `WORKER_MAX_CONCURRENT_JOBS=0` (`D-09`). The Rust worker handles all job processing until complete cutover.
4. **Zero Schema Alteration on Rollback:** Go operates strictly against the existing PostgreSQL table definitions. Rolling back from Go to Rust involves zero database downgrades, zero table drops, and zero data migrations.

---

## 2. Phase P8 Deployment Sequence

### Step 1: Staging Cluster Initialization
Start all staging services with parallel coexistence enabled:
```bash
# Launch full staging topology (PostgreSQL, Redis, MinIO, Rust, Go, Caddy, Prometheus, Grafana)
docker compose -f docker-compose.staging.yml up -d

# Verify both backends report healthy
curl -s http://localhost:3000/api/v1/health | jq .
curl -s http://localhost:3001/api/v1/health | jq .
```

### Step 2: Shadow Traffic Validation (P8-02, P8-03)
Before routing live user traffic to Go, mirror real HTTP traffic from the Rust baseline to Go in safe, read-only mode:
```bash
# Start shadow traffic mirroring daemon (GET/HEAD requests only)
./scripts/shadow_traffic.sh start

# Verify shadow process status
./scripts/shadow_traffic.sh status

# Run automated shadow analysis and verify 0.00% divergence
./scripts/shadow_run_analyzer.py --rust-url http://127.0.0.1:3000 --go-url http://127.0.0.1:3001 --out-json /tmp/shadow_divergence.json
```
**Go / No-Go Gate 1:** Matched requests must equal 100.00% (0 mismatches) on canonical endpoints, with p99 latency delta within 10ms.

### Step 3: Progressive Canary Rollout (P8-04)
Route traffic progressively to the Go service using `scripts/canary_route_manager.sh`:

| Step | Traffic Weight | Duration | Acceptance Criteria |
|---|---|---|---|
| **Phase 1** | **1%** | 2 Hours | 5xx Rate $< 0.1\%$, p99 Latency $< 35\text{ms}$, 0 fatal log errors |
| **Phase 2** | **5%** | 4 Hours | 5xx Rate $< 0.2\%$, p99 Latency $< 35\text{ms}$, DB pool $< 25\%$ |
| **Phase 3** | **25%** | 8 Hours | 5xx Rate $< 0.2\%$, p99 Latency $< 50\text{ms}$, Memory RSS $< 500\text{MB}$ |
| **Phase 4** | **50%** | 12 Hours | 5xx Rate $< 0.2\%$, p99 Latency $< 50\text{ms}$, Memory RSS $< 650\text{MB}$ |
| **Phase 5** | **100%** | 24 Hours | Full traffic on Go candidate; Rust standby ready |

Execution commands:
```bash
# Set 1% canary weight
./scripts/canary_route_manager.sh set-weight 1

# Check routing status
./scripts/canary_route_manager.sh status

# Advance to 5%, 25%, 50%, 100%
./scripts/canary_route_manager.sh set-weight 5
./scripts/canary_route_manager.sh set-weight 25
./scripts/canary_route_manager.sh set-weight 50
./scripts/canary_route_manager.sh set-weight 100
```

### Step 4: Mutating Canary with Isolated Tenants (P8-06)
For write/mutating endpoints (e.g. punches, shift creation, leave approvals), test against designated staging test tenant workspaces:
```bash
# Route requests carrying tenant header to Go:
curl -H "X-Canary-Workspace: ws_staging_tenant_01" http://localhost/api/v1/attendance/punch-in
```

### Step 5: Domain-by-Domain Cutover (P8-09)
Alternatively, cut over specific functional domains individually:
```bash
# Cut over System & Health domain to Go
./scripts/canary_route_manager.sh set-domain system go

# Cut over Employees domain to Go
./scripts/canary_route_manager.sh set-domain employees go

# Cut over Attendance domain to Go
./scripts/canary_route_manager.sh set-domain attendance go
```

---

## 3. Emergency Rollback Standard Operating Procedure (SOP)

### Rollback Thresholds (Automated Triggers)
An automated rollback is triggered immediately if ANY of the following occurs:
1. **5xx Error Rate:** Exceeds **0.5%** of requests over a 1-minute evaluation window.
2. **Latency Breach:** p99 response time exceeds **100ms** over a 2-minute evaluation window.
3. **Readiness Probe Failure:** `/api/v1/health/ready` returns non-200 for 3 consecutive polls (15 seconds).
4. **Database Pool Saturation:** PgBouncer connection utilization exceeds **85%**.
5. **Data Integrity Alarm:** Any detectable divergence in database transactions or audit trails.

### Immediate Rollback Procedure (< 2 Seconds Execution)
Run the automated rollback script:
```bash
./scripts/canary_rollback.sh "Emergency rollback: [DESCRIBE SYMPTOM/ALERT]"
```

What the rollback script executes:
1. Replaces `staging/caddy/canary_weights.conf` with 0% weight.
2. Clears all per-domain cutovers in `staging/caddy/canary_domains.conf`.
3. Signals Caddy via Admin API `POST http://127.0.0.1:2019/load` to atomically re-route 100% of ingress to `apiserver:3000` (Rust baseline).
4. Verifies `/api/v1/health` on `apiserver:3000`.
5. Logs rollback timestamp, operator ID, elapsed duration, and reason to `docs/migration/rust-go/rollback-audit.log`.

**Observed Rollback Time:** $\approx 42\text{ms} - 120\text{ms}$ (Strictly well below the 2000ms RTO).

---

## 4. Failure Drill Matrix & Symptom Playbooks (P8-05, P8-07)

Run the automated drill verification suite anytime:
```bash
./scripts/test_failure_drills.sh
```

| Drill # | Scenario | Expected System Behavior | Triage & Remediation Command |
|---|---|---|---|
| **1** | **Go Container Down** | Caddy detects connection failure, returns 502 or reroutes to Rust; watchdog triggers rollback. | Check logs: `docker compose logs -n 100 api-go`. Roll back: `./scripts/canary_rollback.sh`. |
| **2** | **PostgreSQL Unreachable** | `/api/v1/health/ready` fails immediately with 500; writes rejected safely; no corrupt commits. | Check PgBouncer: `docker compose logs pgbouncer_tx`. Inspect PostgreSQL: `pg_isready`. |
| **3** | **Redis Unreachable** | Token revocation check fails open safely; login rate limiting logs warning; readiness reports degraded. | Check Redis: `docker compose logs redis`. Test ping: `redis-cli ping`. |
| **4** | **Request Timeout** | Server propagates context cancellation; DB query aborted; client receives 504 Gateway Timeout. | Inspect slow queries via PgBouncer / pg_stat_activity. |
| **5** | **Invalid Token / Attack** | Auth middleware rejects request with 401 Unauthorized before invoking service logic. | Verify JWT secret alignment between Rust and Go configs. |
| **6** | **Worker Queue Stall** | Go worker concurrency is 0 in canary (D-09); Rust worker manages queue; no jobs lost. | Check queue depth in Redis: `LLEN worker:jobs:default`. |
| **7** | **Partial Dependency Failure** | Redis down but DB up: read endpoints succeed; cache-dependent mutations fail closed safely. | Fix Redis dependency; restart cache container. |

---

## 5. Automated Watchdog Daemon

To enable 24/7 automated rollback monitoring in staging:
```bash
# Run watchdog in daemon mode (polls readiness and error budget every 5s)
nohup ./scripts/rollback_trigger_watchdog.py --poll-interval 5 > /tmp/watchdog.log 2>&1 &
```

If the Go service degrades, the watchdog automatically invokes `scripts/canary_rollback.sh` without human intervention.
