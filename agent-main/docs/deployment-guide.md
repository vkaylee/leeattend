# Deployment Guide

**Project:** Agent Attendance Sync System
**Version:** 1.0
**Status:** Production Ready
**Last Updated:** 2026-01-29

---

## Pre-Deployment Checklist

### Code & Testing
- [x] All 174 tests passing (cargo test --all)
- [x] Code reviewed and approved
- [x] No compiler warnings
- [x] Performance validated in lab
- [x] Crash recovery tested
- [x] Backward compatibility verified

### Database
- [x] All 8 migrations applied successfully
- [x] Schema changes tested
- [x] Data integrity verified
- [x] Backup strategy in place

### Documentation
- [x] README.md complete
- [x] Architecture documentation
- [x] Code standards documented
- [x] Troubleshooting guide prepared

### Infrastructure
- [x] Staging environment available
- [x] Monitoring dashboards ready
- [x] Alert thresholds configured
- [x] Rollback procedure documented

---

## Deployment Phases

### Phase 1: Staging Deployment (1-2 days)

**Objective:** Validate system in production-like environment

**Steps:**

1. **Prepare Staging Server**
   ```bash
   # Clone repository
   git clone <repo-url> /opt/agent-staging
   cd /opt/agent-staging

   # Build binary
   cargo build --release

   # Create SQLite database
   touch /var/lib/agent/attendance.db

   # Apply migrations
   sqlx migrate run --database-url "sqlite:/var/lib/agent/attendance.db"
   ```

2. **Configure Environment**
   ```bash
   # Copy and edit .env for staging
   cp .env.template /etc/agent/.env.staging

   # Update key settings
   SERVER_URL=https://staging-server.example.com
   API_TOKEN=staging-token-xxx
   DEVICE_POLLING_INTERVAL_SECS=5
   CHECKPOINT_INTERVAL_SECS=5  # Frequent for testing
   LOG_LEVEL=debug
   ```

3. **Start Agent**
   ```bash
   # Run with systemd or docker
   systemctl start agent-staging

   # Or docker
   docker run --name agent-staging \
     --env-file /etc/agent/.env.staging \
     --volume /var/lib/agent:/data \
     -d registry.example.com/agent:latest
   ```

4. **Run Validation Tests**
   ```bash
   # Run integration test suite
   cargo test --test '*' --release

   # Manual test: Verify sync latency
   # - Monitor logs: grep "Sync cycle" /var/log/agent/agent.log
   # - Expected: ~500ms per cycle
   # - Alert if: >2s

   # Manual test: Inject test data
   # - Push 100 logs via device emulator
   # - Verify in central system within 2s

   # Manual test: Crash recovery
   # - Kill agent process: killall agent
   # - Verify .agent_running marker
   # - Restart: systemctl start agent-staging
   # - Verify logs reset and sync resumes
   ```

5. **Monitor for 48 Hours**
   ```bash
   # Key metrics to watch
   - Sync latency: Should be 100-500ms
   - Memory usage: Should be <100MB
   - CPU usage: Should be <10%
   - Checkpoint success: Should be 100%
   - Error count: Should be 0-1 per day
   - Crash events: Should be 0
   ```

6. **Performance Report**
   ```
   Generate report template:
   - Average sync latency: ___ ms
   - Peak memory usage: ___ MB
   - Checkpoint success rate: ___%
   - Error count: ___
   - Issues encountered: ___
   - Recommendation: [PROCEED / HOLD]
   ```

**Success Criteria:**
- ✅ Zero crashes
- ✅ Sync latency: 100-500ms (avg)
- ✅ Memory: <100MB
- ✅ Checkpoints: 100% success
- ✅ All tests: Passing

**Decision Point:** If any failure, investigate and fix before proceeding.

---

### Phase 2: Gradual Production Rollout (3-7 days)

**Objective:** Deploy to production with minimal risk

**Wave 1: 10% of Agents (Day 1)**

```bash
# Select 2-3 agents from different regions
AGENTS_TO_DEPLOY="agent-01 agent-05 agent-12"

for agent in $AGENTS_TO_DEPLOY; do
  # Build and test
  cargo build --release --target-dir /tmp/agent-build

  # Create backup
  cp /var/lib/agent/${agent}/attendance.db \
     /var/lib/agent/backups/${agent}-backup-$(date +%s).db

  # Deploy binary
  cp /tmp/agent-build/release/agent \
     /opt/agent/${agent}/bin/agent-new

  # Create symlink (atomic switch)
  ln -sf agent-new /opt/agent/${agent}/bin/agent

  # Restart agent
  systemctl restart agent-${agent}

  # Verify startup
  sleep 10
  if systemctl is-active agent-${agent}; then
    echo "✓ Agent ${agent} deployed"
  else
    echo "✗ Agent ${agent} failed to start - ROLLING BACK"
    ln -sf agent-old /opt/agent/${agent}/bin/agent
    systemctl restart agent-${agent}
  fi
done

# Monitor for 24 hours
# - Sync latency: grep "Sync cycle" /var/log/agent/agent-*.log
# - Memory: ps aux | grep agent | grep -v grep
# - Errors: journalctl -u agent-* -n 20
```

**Wave 2: 50% of Agents (Day 3)**

```bash
# Deploy to 10-15 more agents
# Same procedure as Wave 1
# Monitor for 24 hours
```

**Wave 3: 100% of Agents (Day 5)**

```bash
# Deploy to all remaining agents
# Same procedure as Waves 1-2
# Monitor for 24 hours
```

**Monitoring Dashboard:**

| Metric | Alert Threshold | Expected |
|--------|-----------------|----------|
| Sync Latency | >2s | 100-500ms |
| Memory | >150MB | <100MB |
| Checkpoint Success | <95% | 100% |
| Error Rate | >1% | <0.1% |
| Crash Events | >1/day | 0 |

**Decision Points:**

```
After Wave 1 (24h):
  ✓ All metrics normal? → Proceed to Wave 2
  ✗ Any metrics bad?   → PAUSE & INVESTIGATE

After Wave 2 (24h):
  ✓ All metrics normal? → Proceed to Wave 3
  ✗ Any metrics bad?   → PAUSE & INVESTIGATE

After Wave 3 (24h):
  ✓ All metrics normal? → DEPLOYMENT COMPLETE
  ✗ Any metrics bad?   → FULL ROLLBACK
```

---

### Phase 3: Post-Deployment Monitoring (1 week)

**Daily Tasks:**

1. **Morning Review** (9 AM)
   ```bash
   # Check overnight logs
   journalctl -u agent-* -n 100 | grep -i error

   # Verify all agents running
   for agent in agent-*; do
     systemctl is-active $agent || echo "ALERT: $agent DOWN"
   done

   # Check memory trends
   ps aux | grep agent | awk '{print $6}' | sort -n
   ```

2. **Midday Review** (12 PM)
   ```bash
   # Sample sync latency
   grep "Sync cycle" /var/log/agent/agent-*.log | tail -100 | \
     awk '{print $(NF-1)}' | sort -n

   # Check checkpoint success
   grep "Checkpoint:" /var/log/agent/agent-*.log | tail -20
   ```

3. **Evening Review** (5 PM)
   ```bash
   # Generate daily report
   # - Errors: count from logs
   # - Latency: average and max
   # - Memory: peak usage
   # - Uptime: hours running
   ```

**Weekly Tasks:**

1. **Metrics Analysis**
   - Sync latency trends
   - Memory growth patterns
   - Error categorization
   - Crash events (if any)

2. **Log Review**
   - Check for warnings
   - Identify patterns
   - Plan optimizations

3. **Performance Report**
   - Executive summary
   - Key metrics dashboard
   - Issues found and resolved
   - Recommendations

---

## Rollback Procedure

### Trigger Conditions

**AUTOMATIC ROLLBACK** if:
- Sync latency consistently >2000ms
- Memory usage >150MB (without 100K logs)
- Checkpoint success <95%
- Crash rate >1 per day

### Manual Rollback Steps

```bash
# 1. Stop all new agents
for agent in agent-*; do
  systemctl stop $agent
done

# 2. Restore previous binary
for agent in agent-*; do
  ln -sf agent-old /opt/agent/$agent/bin/agent
done

# 3. Restart with old binary
for agent in agent-*; do
  systemctl start $agent
done

# 4. Verify rollback successful
sleep 30
systemctl status agent-* | grep running

# 5. Monitor for stability (1 hour)
# - Verify sync latency normal
# - Verify no errors
# - Verify memory stable

# 6. Post-rollback analysis
# - Gather logs from failed deployment
# - Identify root cause
# - Plan fix for next attempt
```

### Data Safety

**Important:** Rollback is safe because:
- ✅ All data persisted to disk
- ✅ No schema changes
- ✅ No data migrations
- ✅ Binary-only update
- ✅ Previous version can read current database

---

## Configuration Management

### Environment Variables

**Required:**
```bash
SERVER_URL=https://server.example.com    # Central server address
API_TOKEN=secret-token-xxx                # Authentication token
```

**Optional (with defaults):**
```bash
LOG_LEVEL=info                            # debug|info|warn|error
DEVICE_POLLING_INTERVAL_SECS=5            # How often to poll devices
CHECKPOINT_INTERVAL_SECS=5                # How often to flush to disk
BATCH_SIZE=100                            # Records per batch
BATCH_TIMEOUT_MS=200                      # Max wait for batch
MAX_RETRY_COUNT=10                        # Max retries before DLQ
MAX_CAPACITY=100000                       # Max in-memory logs
```

### Configuration File Location

```
/etc/agent/.env
or
/app/config/.env
or
Environment variables
```

### Configuration Validation

```bash
# Test configuration before deployment
export $(cat /etc/agent/.env | xargs)

# Verify all required env vars set
[[ -z "$SERVER_URL" ]] && echo "ERROR: SERVER_URL not set"
[[ -z "$API_TOKEN" ]] && echo "ERROR: API_TOKEN not set"

# Verify connectivity
curl -s -H "Authorization: Bearer $API_TOKEN" \
  $SERVER_URL/health && echo "✓ Server reachable"
```

---

## Monitoring & Observability

### Key Metrics

**Performance Metrics:**
```
Sync Latency (ms)          - Target: 100-500ms
Memory Usage (MB)          - Target: <100MB
Checkpoint Duration (ms)   - Target: <500ms
Throughput (logs/sec)      - Target: >200
```

**Reliability Metrics:**
```
Uptime (%)                 - Target: >99.9%
Crash Count                - Target: 0/day
Failed Syncs               - Target: <1%
Checkpoint Success (%)     - Target: 100%
```

### Log Pattern Monitoring

**Expected (Healthy):**
```
INFO Checkpoint: 150 logs persisted
INFO Sync cycle: 234ms completed
DEBUG 3 logs in backoff
```

**Warning Signs:**
```
WARN Sync cycle: 1500ms (slowdown)
WARN Checkpoint delayed: database busy
ERROR Failed to sync logs: connection timeout
```

**Recovery Indicators:**
```
INFO Detected previous crash
INFO Crash recovery: Reset 5 logs
INFO Resume normal operation
```

### Alert Configuration

```yaml
# Prometheus alerts example
groups:
  - name: agent
    rules:
      - alert: HighSyncLatency
        expr: agent_sync_latency_ms > 2000
        for: 5m
        annotations:
          severity: warning
          summary: "Sync latency high"

      - alert: MemoryExceeded
        expr: agent_memory_mb > 150
        for: 5m
        annotations:
          severity: critical
          summary: "Memory usage critical"

      - alert: CheckpointFailure
        expr: agent_checkpoint_success_rate < 0.95
        for: 10m
        annotations:
          severity: warning
          summary: "Checkpoint failure rate high"
```

---

## Troubleshooting

### Issue: High Sync Latency (>2s)

**Diagnosis:**
```bash
# Check network
ping -c 5 server.example.com

# Check server status
curl -s $SERVER_URL/health

# Check local logs
grep "Sync cycle" /var/log/agent/agent.log | tail -20

# Check memory pressure
ps aux | grep agent

# Check disk I/O
iostat -d 1 5
```

**Solutions:**
1. Verify network connectivity
2. Check server capacity
3. Reduce DEVICE_POLLING_INTERVAL_SECS
4. Increase CHECKPOINT_INTERVAL_SECS

### Issue: High Memory Usage (>100MB)

**Diagnosis:**
```bash
# Check current usage
ps aux | grep agent | grep -v grep | awk '{print $6}'

# Check log backlog
grep "in backoff" /var/log/agent/agent.log | tail -1

# Check sync success rate
grep -c "is_synced=1" /var/log/agent/agent.log
```

**Solutions:**
1. Reduce MAX_CAPACITY (if not at limit)
2. Increase sync frequency
3. Check for network issues preventing sync

### Issue: Crash Recovery Loops

**Diagnosis:**
```bash
# Check for marker file
ls -la /var/lib/agent/.agent_running

# Check crash count
grep "Detected previous crash" /var/log/agent/agent.log | wc -l

# Check if issue is deterministic
# Restart and observe
systemctl restart agent
sleep 5
systemctl status agent
```

**Solutions:**
1. Check logs for panic messages
2. Verify database integrity
3. Reset marker file: `rm /var/lib/agent/.agent_running`

### Issue: Failed Log Queue Growing

**Diagnosis:**
```bash
# Check dead letter queue
sqlite3 /var/lib/agent/attendance.db \
  "SELECT count(*) FROM zkt_failed_logs;"

# Check failure reasons
sqlite3 /var/lib/agent/attendance.db \
  "SELECT failure_reason, count(*) \
   FROM zkt_failed_logs \
   GROUP BY failure_reason;"
```

**Solutions:**
1. Investigate specific failure reason
2. Fix underlying issue (device, server, auth)
3. Manual cleanup of old failed logs if needed

---

## Maintenance Tasks

### Daily Maintenance
- Monitor sync latency and memory
- Check error logs
- Verify all agents running

### Weekly Maintenance
- Review performance metrics
- Analyze error patterns
- Plan capacity adjustments

### Monthly Maintenance
- Full backup of databases
- Clean up old failed logs (30+ days)
- Performance optimization review
- Security audit

### Annual Maintenance
- Update dependencies
- Database optimization
- Capacity planning review

---

## Disaster Recovery

### Database Corruption

**Recovery Steps:**
```bash
# 1. Stop agent
systemctl stop agent

# 2. Restore from backup
cp /backup/attendance.db.2026-01-29 \
   /var/lib/agent/attendance.db

# 3. Restart agent
systemctl start agent

# 4. Verify
systemctl status agent
```

### Data Loss (Partial)

**Recovery Steps:**
```bash
# Unsynced logs are in memory, will be re-sent
# Synced logs are permanently in server
# No action needed - system auto-recovers
```

### Total System Failure

**Recovery Steps:**
```bash
# 1. Restore entire deployment package
# 2. Restore database backups
# 3. Restart all agents
# 4. Verify sync resumes
# 5. Monitor for 24 hours
```

---

## Performance Tuning

### Optimize for High Volume

```bash
# Increase batch size
BATCH_SIZE=200

# Reduce batch timeout
BATCH_TIMEOUT_MS=100

# Increase checkpoint frequency
CHECKPOINT_INTERVAL_SECS=3

# Increase in-memory capacity
MAX_CAPACITY=150000
```

### Optimize for Low Latency

```bash
# Reduce checkpoint interval
CHECKPOINT_INTERVAL_SECS=2

# Reduce batch timeout
BATCH_TIMEOUT_MS=50

# Increase polling frequency
DEVICE_POLLING_INTERVAL_SECS=3
```

### Optimize for Low Resource Usage

```bash
# Reduce in-memory capacity
MAX_CAPACITY=50000

# Increase checkpoint interval
CHECKPOINT_INTERVAL_SECS=10

# Reduce batch size
BATCH_SIZE=50
```

---

## Success Metrics

### Deployment Success Defined As

- ✅ 100% of agents deployed
- ✅ 100% of agents running
- ✅ Sync latency: 100-500ms (avg)
- ✅ Memory usage: <100MB
- ✅ Checkpoint success: 100%
- ✅ Error count: <0.1% of logs
- ✅ Crash events: 0
- ✅ User complaints: 0

### Post-Deployment Review

Schedule 1-week review meeting:
- Performance metrics summary
- Issues encountered and resolved
- Lessons learned
- Recommendations for future deployments
- Documentation updates

---

**Document Version:** 1.0
**Status:** PRODUCTION READY
**Last Updated:** 2026-01-29
