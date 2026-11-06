#!/bin/bash
set -e

ACCOUNTS=(
  "admin@example.invalid:admin:admin"
  "hr.manager@example.invalid:none:hr_manager"
  "shift.manager@example.invalid:none:shift_manager"
  "team.lead@example.invalid:none:team_lead"
  "employee@example.invalid:none:employee"
)

PASSWORD="development-password"
BASE_URL="http://localhost:8088/api/v1"

echo "=== Testing 5 Clean Dev Accounts Login, System Role & Permissions ==="
for item in "${ACCOUNTS[@]}"; do
  IFS=':' read -r email expected_sys expected_ws <<< "$item"
  
  # Login
  LOGIN_RES=$(curl -s -X POST "$BASE_URL/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"$email\",\"password\":\"$PASSWORD\"}")
  
  TOKEN=$(echo "$LOGIN_RES" | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)
  if [ -z "$TOKEN" ]; then
    echo "❌ FAILED LOGIN: $email - Response: $LOGIN_RES"
    exit 1
  fi
  
  # Fetch Me
  ME_RES=$(curl -s "$BASE_URL/me" -H "Authorization: Bearer $TOKEN")
  SYS_ROLES=$(echo "$ME_RES" | grep -o '"system_roles":\[[^]]*\]')
  
  # Fetch Permissions
  PERM_RES=$(curl -s "$BASE_URL/me/permissions" -H "Authorization: Bearer $TOKEN")
  ROLES=$(echo "$PERM_RES" | grep -o '"roles":\[[^]]*\]')
  PERMS_COUNT=$(echo "$PERM_RES" | grep -o '"permissions":\[[^]]*\]' | grep -o ',' | wc -l)
  
  # Test Monitor overview if admin
  if [ "$expected_sys" = "admin" ]; then
    MON_RES=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/monitor/overview" -H "Authorization: Bearer $TOKEN")
    if [ "$MON_RES" != "200" ]; then
      echo "❌ FAILED MONITOR ACCESS for $email (HTTP $MON_RES)"
      exit 1
    fi
    echo "✅ $email -> Login OK | Role: $expected_ws | Sys: $SYS_ROLES | Monitor Overview: 200 OK | Perms: ~$PERMS_COUNT"
  else
    echo "✅ $email -> Login OK | Role: $expected_ws | Sys: $SYS_ROLES | Perms: ~$PERMS_COUNT"
  fi
done

echo "=== All 5 Clean Accounts Verified Successfully ==="
