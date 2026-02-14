# Fix: Test Container Isolation

**Date**: 2026-02-14
**Type**: Bug Fix
**Severity**: High
**Status**: Fixed

## Problem

Test containers were using the default Docker Compose project name (based on directory name), causing cleanup operations to potentially affect containers from other compose files:

- `docker-compose.yml` (production)
- `docker-compose.dev.yml` (development)
- `docker-compose.test.yml` (testing)

All three files in the same directory (`leeattend`) would share the project name `leeattend`. When running cleanup with `--remove-orphans`, containers from dev/prod could be accidentally removed.

## Impact

**Before Fix:**
```bash
# This could affect containers from ALL compose files
docker compose -f docker-compose.test.yml down --remove-orphans
```

**Risk:**
- Development containers (`db`, `redis`, `apiserver`) could be stopped
- Production containers could be affected if deployed locally
- Data loss if volumes accidentally removed

## Solution

Added explicit project name `leeattend-test` for all test container operations.

### Changes Made

#### 1. `run-tests.sh`
```bash
# Before
export DOCKER_COMPOSE_CMD="docker compose -f $PROJECT_ROOT/docker-compose.test.yml"

# After
export DOCKER_COMPOSE_CMD="docker compose -p leeattend-test -f $PROJECT_ROOT/docker-compose.test.yml"
```

#### 2. `setup-test-env.sh`
```bash
# Added
export COMPOSE_PROJECT_NAME="leeattend-test"

# Updated all commands
docker compose -p "$COMPOSE_PROJECT_NAME" -f "$COMPOSE_FILE" [command]
```

## Verification

### Container Naming
- **Test containers**: `leeattend-test-test_db-1`, `leeattend-test-apiserver_test-1`
- **Dev containers**: `leeattend-db-1`, `leeattend-apiserver-1`
- **Prod containers**: `leeattend-db-1`, `leeattend-apiserver-1`

### Test Isolation
```bash
# Start dev containers
docker compose -f docker-compose.dev.yml up -d

# Run tests (isolated)
./run-tests.sh --api

# Dev containers remain running ✓
docker compose -f docker-compose.dev.yml ps
# STATUS: Up
```

## Benefits
- ✅ Test cleanup only affects test containers
- ✅ Dev/prod containers protected from accidental removal
- ✅ Clear naming convention for container identification
- ✅ Safe to run tests alongside dev environment

## Testing Commands

```bash
# Check project names
docker ps --format "table {{.Names}}\t{{.Image}}" | grep leeattend

# Test cleanup (should only remove test containers)
./run-tests.sh --clean

# Verify dev containers unaffected
docker compose -f docker-compose.dev.yml ps
```

## Related Files
- `run-tests.sh` - Main test runner
- `setup-test-env.sh` - Test environment setup
- `docker-compose.test.yml` - Test containers definition

## Notes
- Project name flag (`-p`) takes precedence over directory name
- Docker Compose uses project name for container isolation
- `--remove-orphans` now safely scoped to test project only
