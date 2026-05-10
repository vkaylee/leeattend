#!/bin/sh
# scripts/init-supavisor.sh

set -e

# Wait for Supavisor API to be ready
echo "Waiting for Supavisor API on port 4000..."
max_retries=30
count=0
while ! nc -z supavisor 4000; do
  sleep 1
  count=$((count+1))
  if [ $count -ge $max_retries ]; then
    echo "Timeout waiting for Supavisor API"
    exit 1
  fi
done

echo "Supavisor API is reachable. Registering tenant 'leeattend'..."

# Add curl if it doesn't exist (alpine base image might not have it installed depending on the tag, but we can just use apk)
if ! command -v curl >/dev/null 2>&1; then
  apk add --no-cache curl
fi

# The payload to register the tenant
PAYLOAD=$(cat <<EOF
{
  "tenant_id": "leeattend",
  "db_host": "db",
  "db_port": 5432,
  "db_database": "${DB_NAME}",
  "db_user": "${DB_USER}",
  "db_password": "${DB_PASSWORD}",
  "pool_size": 20,
  "max_clients": 500
}
EOF
)

# Call Supavisor API to register tenant
# Supavisor will return 200/201 if successful or 409 if it already exists
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST http://supavisor:4000/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD")

HTTP_STATUS=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_STATUS" = "200" ] || [ "$HTTP_STATUS" = "201" ]; then
  echo "✅ Tenant 'leeattend' registered successfully: $BODY"
elif [ "$HTTP_STATUS" = "409" ]; then
  echo "✅ Tenant 'leeattend' already exists."
else
  echo "❌ Failed to register tenant. HTTP Status: $HTTP_STATUS"
  echo "Response: $BODY"
  exit 1
fi

echo "Supavisor init completed."
