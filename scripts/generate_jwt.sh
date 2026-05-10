#!/bin/bash
secret="your-super-secret-jwt-token-with-at-least-32-bytes-long"
header='{"typ":"JWT","alg":"HS256"}'
payload='{"iat": 1600000000, "exp": 1900000000}'
b64enc() { openssl enc -base64 -A | tr '+/' '-_' | tr -d '='; }
header_b64=$(echo -n "$header" | b64enc)
payload_b64=$(echo -n "$payload" | b64enc)
sig=$(echo -n "$header_b64.$payload_b64" | openssl dgst -sha256 -hmac "$secret" -binary | b64enc)
echo "$header_b64.$payload_b64.$sig"
