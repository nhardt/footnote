#!/bin/bash
set -e

# Cleanup and setup
rm -rf /tmp/footnotetest
mkdir -p /tmp/footnotetest
cd /tmp/footnotetest

# Create Alice primary device
mkdir alice-desktop && cd alice-desktop
echo "Creating Alice primary device..."
footnote-cli vault create-primary alice alice-desktop
cd ..

# Create Alice secondary device (standalone, ready to join)
mkdir alice-phone && cd alice-phone
echo "Creating Alice secondary device (standalone)..."
footnote-cli vault create-standalone

echo "Secondary device listening for join..."
timeout 30 footnote-cli service join-listen > /tmp/join_listen_output.txt 2>&1 &
JOIN_LISTEN_PID=$!
cd ..

# Wait for connection URL to appear
echo "Waiting for connection URL..."
sleep 3
echo "Extracting connection url..."
cat /tmp/join_listen_output.txt

# Extract connection URL from output
CONNECTION_URL=$(cat /tmp/join_listen_output.txt | jq -r '.join_url')

if [ -z "$CONNECTION_URL" ] || [ "$CONNECTION_URL" = "null" ]; then
    echo "ERROR: Could not capture connection URL"
    cat /tmp/join_listen_output.txt
    kill $JOIN_LISTEN_PID 2>/dev/null || true
    exit 1
fi

echo "Connection URL: $CONNECTION_URL"

# Primary device joins the secondary
cd alice-desktop
echo "Primary adding secondary device to group..."
footnote-cli service join "$CONNECTION_URL" alice-phone
cd ..

wait $JOIN_LISTEN_PID 2>/dev/null || true

echo "Pairing complete"
echo ""
echo "Verifying setup..."

# Verify primary contact.json
PRIMARY_CONTACT=$(cat alice-desktop/.footnote/user.json)
PRIMARY_DEVICES=$(echo "$PRIMARY_CONTACT" | jq '.devices | length')
PRIMARY_SIGNATURE=$(echo "$PRIMARY_CONTACT" | jq -r '.signature')

echo "Primary devices: $PRIMARY_DEVICES"
echo "Primary signature: ${PRIMARY_SIGNATURE:0:20}..."

test "$PRIMARY_DEVICES" -eq 2 || { echo "ERROR: Primary should have 2 devices, got $PRIMARY_DEVICES"; exit 1; }
test -n "$PRIMARY_SIGNATURE" || { echo "ERROR: Primary signature is empty"; exit 1; }

# Verify secondary device contact.json
PHONE_CONTACT=$(cat alice-phone/.footnote/user.json)
PHONE_DEVICES=$(echo "$PHONE_CONTACT" | jq '.devices | length')
PHONE_SIGNATURE=$(echo "$PHONE_CONTACT" | jq -r '.signature')

echo "Phone devices: $PHONE_DEVICES"
echo "Phone signature: ${PHONE_SIGNATURE:0:20}..."

test "$PHONE_DEVICES" -eq 2 || { echo "ERROR: Phone should have 2 devices, got $PHONE_DEVICES"; exit 1; }
test -n "$PHONE_SIGNATURE" || { echo "ERROR: Phone signature is empty"; exit 1; }

# Verify signatures match (both should have identical contact.json)
if [ "$PRIMARY_SIGNATURE" != "$PHONE_SIGNATURE" ]; then
    echo "ERROR: Signatures do not match"
    echo "Primary: $PRIMARY_SIGNATURE"
    echo "Phone: $PHONE_SIGNATURE"
    exit 1
fi

echo "Signatures match!"

# Verify device names in contact
DEVICE1=$(echo "$PRIMARY_CONTACT" | jq -r '.devices[0].name')
DEVICE2=$(echo "$PRIMARY_CONTACT" | jq -r '.devices[1].name')

echo "Device 1: $DEVICE1"
echo "Device 2: $DEVICE2"

# Cleanup
rm -f /tmp/join_listen_output.txt

echo ""
echo "Device join test passed!"
