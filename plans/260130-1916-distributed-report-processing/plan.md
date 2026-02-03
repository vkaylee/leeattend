---
title: "Distributed Attendance Report Processing System"
description: "Queue-based micro-batching architecture for large-scale attendance report generation"
status: completed
priority: P1
effort: 24h
branch: main
tags: [architecture, redis, distributed, reports, performance]
created: 2026-01-30
completed: 2026-01-30
---

# Distributed Attendance Report Processing System

## Overview

Design a distributed report processing system to handle attendance reports for 7,000+ employees across thousands of containers with limited resources (256MB RAM minimum).

**Problem**: Current synchronous report generation loads all employees into memory, causing OOM crashes and blocking API requests during large exports.

**Solution**: Redis-based job queue with micro-batching (20 employees/job), semaphore-controlled workers, and S3-backed result aggregation.

## Architecture Diagram

```
                                   ┌─────────────────────────────────────────────────────────────────┐
                                   │                    DISTRIBUTED REPORT SYSTEM                     │
                                   └─────────────────────────────────────────────────────────────────┘

┌──────────────┐    ┌─────────────────────┐    ┌─────────────────────────────────────────────────────────┐
│   Client     │───▶│   API Server        │───▶│  Redis Queue Infrastructure                             │
│  Dashboard   │    │  (Coordinator)      │    │                                                          │
└──────────────┘    └─────────────────────┘    │  ┌─────────────────┐  ┌─────────────────┐               │
       ▲                    │                   │  │ report:jobs      │  │ report:results   │               │
       │                    │ Create JobID      │  │ (LIST/STREAM)   │  │ (HASH)           │               │
       │                    │ Split employees   │  └────────┬────────┘  └────────┬────────┘               │
       │                    ▼                   │           │                    │                        │
       │            ┌─────────────────┐        │           ▼                    ▼                        │
       │            │ Job Metadata     │        │  ┌─────────────────────────────────────────┐            │
       │            │ report:meta:{id} │        │  │           CONTAINER WORKERS             │            │
       │            └─────────────────┘        │  │                                          │            │
       │                                        │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐   │            │
       │            Status Polling              │  │  │Worker 1 │ │Worker 2 │ │Worker N │   │            │
       └────────────────────────────────────────│──│  │BLPOP    │ │BLPOP    │ │BLPOP    │   │            │
                                               │  │  │Semaphore│ │Semaphore│ │Semaphore│   │            │
                                               │  │  └────┬────┘ └────┬────┘ └────┬────┘   │            │
                                               │  │       │          │           │         │            │
                                               │  └───────┼──────────┼───────────┼─────────┘            │
                                               │          ▼          ▼           ▼                      │
                                               │  ┌─────────────────────────────────────────┐            │
                                               │  │     PostgreSQL (Read)                   │            │
                                               │  │     - raw_attendance_logs               │            │
                                               │  │     - time_entries                      │            │
                                               │  │     - employees (paginated batch)       │            │
                                               │  └─────────────────────────────────────────┘            │
                                               └─────────────────────────────────────────────────────────┘
                                                          │
                                                          ▼
                                               ┌─────────────────────┐    ┌─────────────────┐
                                               │ Result Aggregation  │───▶│    S3 Bucket    │
                                               │ (MERGE Job)         │    │ Excel/CSV Files │
                                               └─────────────────────┘    └─────────────────┘
```

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Queue Backend | Redis LIST + BLPOP | Low latency, atomic operations, no extra infra |
| Batch Size | 20 employees/job | ~40MB RAM per job (safe for 256MB containers) |
| Concurrency Control | Tokio Semaphore | Per-container limit, no coordination needed |
| Result Storage | Redis HASH + TTL | Fast aggregation, auto-expiry after 24h |
| File Export | S3 presigned URL | Offload storage, client-side download |
| DB Strategy | Application-Side Join | Reduce DB CPU, leverage Redis cache for shifts |

## Phases

| Phase | Title | Status | Effort |
|-------|-------|--------|--------|
| 1 | [Redis Queue Infrastructure](./phase-01-redis-queue-infrastructure.md) | pending | 4h |
| 2 | [Coordinator Service](./phase-02-coordinator-service.md) | pending | 4h |
| 3 | [Generic Worker Pattern](./phase-03-generic-worker.md) | pending | 5h |
| 4 | [Application-Side Join](./phase-04-application-side-join.md) | pending | 4h |
| 5 | [Result Aggregation](./phase-05-result-aggregation.md) | pending | 4h |
| 6 | [API Integration](./phase-06-api-integration.md) | pending | 3h |

## Dependencies

- Redis 6.0+ (for STREAMS if needed, LIST otherwise)
- S3-compatible storage (AWS S3, MinIO, or R2)
- Existing PostgreSQL database
- Existing Tokio async runtime

## Acceptance Criteria

1. No OOM crash on 256MB RAM containers
2. DB CPU < 80% with 1,000 concurrent containers
3. Export time scales linearly with container count
4. API response < 200ms for job submission
5. Status polling < 50ms response time

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Redis unavailable | Low | High | Fallback to sync processing for small reports |
| Job stuck/orphaned | Medium | Medium | TTL + heartbeat + dead letter queue |
| Memory spike during merge | Low | High | Streaming aggregation, chunked Excel writing |
| Network partition | Low | Medium | Idempotent jobs, at-least-once delivery |

## Related Files

- `/home/elt1541/leeattend/apiserver/src/services/report_service.rs`
- `/home/elt1541/leeattend/apiserver/src/services/attendance_calculator.rs`
- `/home/elt1541/leeattend/apiserver/src/services/agent_log_buffer.rs` (worker pattern reference)
- `/home/elt1541/leeattend/plans/reports/scout-260130-1711-attendance-report-architecture.md`
