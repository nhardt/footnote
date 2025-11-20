#!/bin/bash
set -e

# Cleanup and setup
rm -rf /tmp/fieldnotest
mkdir -p /tmp/fieldnotest
cd /tmp/fieldnotest

# Create Alice HQ and capture JSON output
mkdir alice-hq && cd alice-hq
OUTPUT=$(fieldnote hq create --device-name alice-desktop)

# Parse JSON with jq
VAULT_PATH=$(echo "$OUTPUT" | jq -r '.vault_path')
MASTER_KEY=$(echo "$OUTPUT" | jq -r '.master_public_key')
DEVICE_NAME=$(echo "$OUTPUT" | jq -r '.device_name')
ENDPOINT_ID=$(echo "$OUTPUT" | jq -r '.device_endpoint_id')

# Verify output
echo "Vault created at: $VAULT_PATH"
echo "Master key: $MASTER_KEY"
echo "Device name: $DEVICE_NAME"
echo "Endpoint ID: $ENDPOINT_ID"

# Verify files exist
test -d .fieldnotes || { echo "ERROR: .fieldnotes not found"; exit 1; }
test -f .fieldnotes/this_device || { echo "ERROR: this_device key not found"; exit 1; }
test -f .fieldnotes/master_identity || { echo "ERROR: master_identity not found"; exit 1; }
test -f .fieldnotes/contact.json || { echo "ERROR: contact.json not found"; exit 1; }
test -f identity.md || { echo "ERROR: identity.md not found"; exit 1; }

# Verify contact.json content
CONTACT_JSON=$(cat .fieldnotes/contact.json)
CONTACT_USERNAME=$(echo "$CONTACT_JSON" | jq -r '.username')
CONTACT_SIGNATURE=$(echo "$CONTACT_JSON" | jq -r '.signature')
CONTACT_DEVICES=$(echo "$CONTACT_JSON" | jq '.devices | length')

echo "Contact username: $CONTACT_USERNAME"
echo "Contact signature: ${CONTACT_SIGNATURE:0:20}..."
echo "Contact devices: $CONTACT_DEVICES"

# Verify signature exists
test -n "$CONTACT_SIGNATURE" || { echo "ERROR: contact signature is empty"; exit 1; }
test "$CONTACT_DEVICES" -eq 1 || { echo "ERROR: expected 1 device, got $CONTACT_DEVICES"; exit 1; }

echo "HQ create test passed!"
