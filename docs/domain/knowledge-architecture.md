# LeeAttend — System Architecture

## Runtime topology

```text
[React/Vite dashboard] -- HTTP/WebSocket --> [Go apiserver]
                                      |-- SQL --> [PostgreSQL]
                                      |-- queue/pubsub --> [Redis]
                                      |-- objects --> [MinIO]
```

LeeAttend runs as Go `apiserver` plus React/Vite/TypeScript dashboard. PostgreSQL stores organization, attendance, payroll, and audit data. Redis handles rate limiting, queues, and report progress. MinIO stores uploaded files.

## API layers

1. HTTP handlers validate requests, enforce authentication and permissions, and map responses.
2. Services coordinate business workflows and transaction boundaries.
3. Domain packages hold deterministic attendance, scheduling, and payroll rules.
4. Repositories execute parameterized PostgreSQL queries through `pgx`.
5. Migrations define the database schema and constraints.

## Data flow

```text
Attendance source → Go apiserver → raw attendance logs
                                 ↓
                         punch pairing and policy resolution
                                 ↓
                         daily attendance summaries
                                 ↓
                    reports, overtime, and payroll calculations
```

## Security boundaries

- Authentication uses JWT and refresh-token flow.
- Authorization uses explicit user roles and permission checks.
- Organization scope comes from trusted server context; clients cannot select scope.
- Logs, metrics, traces, and error payloads omit secrets and raw PII.
- Public-facing API inputs receive UUID, size, date, and domain validation.

## Verification

```bash
./scripts/_compose-validate.sh
leedevkit test api
leedevkit test web
leedevkit test all
```
