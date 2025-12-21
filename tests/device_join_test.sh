#!/bin/bash
set -e

# Cleanup and setup
rm -rf /tmp/footnotetest
mkdir -p /tmp/footnotetest
cd /tmp/footnotetest

# Create Alice primary device
mkdir alice-primary && cd alice-primary
echo "Creating Alice primary device..."
footnote-cli init --username alice --device-name desktop > /dev/null 2>&1
cd ..

# Start device authorization in background and capture connection URL
echo "Starting device authorization..."
cd alice-primary
timeout 30 footnote-cli device create > /tmp/device_create_output.txt 2>&1 &
DEVICE_PID=$!
cd ..

# Wait for connection URL to appear
echo "Waiting for connection URL..."
sleep 3

# Extract connection URL from output
CONNECTION_URL=$(grep -o 'iroh://[^[:space:]]*' /tmp/device_create_output.txt | head -1)

if [ -z "$CONNECTION_URL" ]; then
    echo "ERROR: Could not capture connection URL"
    cat /tmp/device_create_output.txt
    kill $DEVICE_PID 2>/dev/null || true
    exit 1
fi

echo "Connection URL: $CONNECTION_URL"

# Create Alice secondary device and join
mkdir alice-phone && cd alice-phone
echo "Creating Alice secondary device..."
footnote-cli mirror from "$CONNECTION_URL" --device-name phone > /dev/null 2>&1
cd ..

# Wait for background process to finish
wait $DEVICE_PID 2>/dev/null || true

echo ""
echo "Verifying setup..."

# Verify primary contact.json
PRIMARY_CONTACT=$(cat alice-primary/.footnotes/contact.json)
PRIMARY_DEVICES=$(echo "$PRIMARY_CONTACT" | jq '.devices | length')
PRIMARY_SIGNATURE=$(echo "$PRIMARY_CONTACT" | jq -r '.signature')

echo "Primary devices: $PRIMARY_DEVICES"
echo "Primary signature: ${PRIMARY_SIGNATURE:0:20}..."

test "$PRIMARY_DEVICES" -eq 2 || { echo "ERROR: Primary should have 2 devices, got $PRIMARY_DEVICES"; exit 1; }
test -n "$PRIMARY_SIGNATURE" || { echo "ERROR: Primary signature is empty"; exit 1; }

# Verify secondary device contact.json
PHONE_CONTACT=$(cat alice-phone/.footnotes/contact.json)
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
DEVICE1=$(echo "$PRIMARY_CONTACT" | jq -r '.devices[0].device_name')
DEVICE2=$(echo "$PRIMARY_CONTACT" | jq -r '.devices[1].device_name')

echo "Device 1: $DEVICE1"
echo "Device 2: $DEVICE2"

# Cleanup
rm -f /tmp/device_create_output.txt

echo ""
echo "Device join test passed!"
