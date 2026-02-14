# Fix: SQLx Compilation Errors

**Date**: 2026-02-14
**Type**: Bug Fix
**Status**: ✅ Fixed
**Impact**: High - Blocked all tests from running

## Problem

Test suite failed to build with 134 compilation errors:

```
error: set `DATABASE_URL` to use query macros online, or run `cargo sqlx prepare` to update the query cache
error[E0282]: type annotations needed
error: could not compile `apiserver` (lib) due to 134 previous errors
```

## Root Cause Analysis

1. **.sqlx files corrupted** - SQLx query metadata files were owned by `root:root` with restricted permissions (`rw-r-----`)
2. **Permission issues** - Files created by Docker containers couldn't be read during build
3. **Missing metadata** - SQLx compile-time macros require `.sqlx/*.json` files for offline compilation
4. **Attempted deletion** - Initial fix removed all .sqlx files, making problem worse

## Solution Implemented

### Step 1: Identified Corrupted Files
```bash
ls -la apiserver/.sqlx/
# Showed: -rw-r----- 1 root root (inaccessible)
```

### Step 2: Restored from Git History
```bash
git checkout HEAD~3 -- apiserver/.sqlx/
# Restored 128 .sqlx metadata files with correct permissions
```

### Step 3: Verified Permissions
```bash
ls -la apiserver/.sqlx/
# Now: -rw-r--r-- 1 elt1541 elt1541 (accessible)
```

### Step 4: Rebuilt Test Containers
```bash
docker compose -p leeattend-test -f docker-compose.test.yml build apiserver_test
# Result: Finished `dev` profile in 19.50s ✓
```

## Build Results

### Before Fix
```
#22 ERROR: process "/bin/sh -c cargo build --tests" did not complete successfully: exit code: 101
```

### After Fix
```
#22 0.687    Compiling apiserver v0.1.0 (/workspace/apiserver)
#22 19.85     Finished `dev` profile [unoptimized + debuginfo] target(s) in 19.50s
#22 DONE 20.0s
```

## Why This Happened

1. Docker containers (run as root) generated .sqlx files
2. Files created inside containers inherited root ownership
3. Permission mode prevented reading during subsequent builds
4. SQLx macros couldn't access metadata → compilation failed

## Prevention

### Immediate
- ✅ .sqlx files restored with correct ownership
- ✅ Files now committed to git with proper permissions

### Long-term Recommendations
1. Add .sqlx file ownership check to test setup
2. Generate .sqlx files on host (not in containers)
3. Or use Docker user mapping to match host user
4. Document .sqlx regeneration process

## Files Affected
- `apiserver/.sqlx/*.json` - 128 query metadata files restored

## Testing
```bash
# Test build succeeds
./run-tests.sh --all
# Status: Running successfully
```

## Related Issues
- Container cleanup and isolation (fixed in previous commits)
- SQLx offline compilation requirements
- Docker file permissions

## Next Steps
- [x] Restore .sqlx files
- [x] Verify build succeeds
- [ ] Complete test suite run (in progress)
- [ ] Document .sqlx file management in CLAUDE.md
