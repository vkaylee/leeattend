# Implementation Plan: NSSM Service Wrapper for API Server

**Task:** Pack the Rust `apiserver` into a Windows System Service using NSSM.

## Approach
- **Why:** Consistency with the `agent` implementation, high reliability, automatic restarts, and easy log management.
- **Tool:** NSSM (Non-Sucking Service Manager).

## Steps

### 1. Prepare Binary (5 min)
- Ensure `apiserver` is built in release mode.
- Location: `apiserver/target/release/leeattend_apiserver.exe` (or similar name).

### 2. Copy NSSM (2 min)
- Copy `nssm.exe` from `agent/` to `apiserver/` (if it exists) or instruct user to download it.

### 3. Create Install Script (15 min)
- Create `apiserver/install_service.bat`.
- Logic:
    - Check for Admin rights.
    - Set `SERVICE_NAME=LeeAttendAPIServer`.
    - Point to the Rust `.exe`.
    - Configure working directory and log rotation.

### 4. Create Uninstall Script (5 min)
- Create `apiserver/uninstall_service.bat`.
- Logic: Stop and remove the service.

### 5. Verification (10 min)
- Run `install_service.bat`.
- Check `services.msc` to ensure it's running.
- Verify logs in `apiserver/logs/`.

## Implementation Details

### `apiserver/install_service.bat`
```batch
@echo off
set SERVICE_NAME=LeeAttendAPIServer
set BASE_DIR=%~dp0
set NSSM_EXE=%BASE_DIR%nssm.exe
set APP_EXE=%BASE_DIR%target\release\leeattend_apiserver.exe
set LOG_DIR=%BASE_DIR%logs

if not exist "%LOG_DIR%" mkdir "%LOG_DIR%"

"%NSSM_EXE%" install %SERVICE_NAME% "%APP_EXE%"
"%NSSM_EXE%" set %SERVICE_NAME% AppDirectory "%BASE_DIR%"
"%NSSM_EXE%" set %SERVICE_NAME% AppStdout "%LOG_DIR%\stdout.log"
"%NSSM_EXE%" set %SERVICE_NAME% AppStderr "%LOG_DIR%\stderr.log"
"%NSSM_EXE%" start %SERVICE_NAME%
```

## Timeline
| Phase | Duration |
|-------|----------|
| Preparation | 7 min |
| Scripting | 20 min |
| Testing | 10 min |
| **Total** | **37 min** |

## Rollback Plan
1. Run `uninstall_service.bat`.
2. Delete the created `.bat` files.

## Security Checklist
- [ ] Ensure logs don't contain sensitive ENV data.
- [ ] Run as LocalSystem or a dedicated service account.
- [ ] Firewall rules: Ensure API port is open for Agent connections.
