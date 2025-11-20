#!/bin/bash
set -e

# Cleanup and setup
rm -rf /tmp/fieldnotest
mkdir -p /tmp/fieldnotest
cd /tmp/fieldnotest

# Create Alice HQ
mkdir alice-hq && cd alice-hq
echo "Creating Alice HQ..."
fieldnote hq create --username alice --device-name desktop > /dev/null 2>&1
cd ..

# Start device authorization in background and capture connection URL
echo "Starting device authorization..."
cd alice-hq
timeout 30 fieldnote device create > /tmp/device_create_output.txt 2>&1 &
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

# Create Alice outpost and join
mkdir alice-outpost && cd alice-outpost
echo "Creating Alice outpost..."
fieldnote device create remote "$CONNECTION_URL" --device-name phone > /dev/null 2>&1
cd ..

# Wait for background process to finish
wait $DEVICE_PID 2>/dev/null || true

echo ""
echo "Verifying setup..."

# Verify HQ contact.json
HQ_CONTACT=$(cat alice-hq/.fieldnotes/contact.json)
HQ_DEVICES=$(echo "$HQ_CONTACT" | jq '.devices | length')
HQ_SIGNATURE=$(echo "$HQ_CONTACT" | jq -r '.signature')

echo "HQ devices: $HQ_DEVICES"
echo "HQ signature: ${HQ_SIGNATURE:0:20}..."

test "$HQ_DEVICES" -eq 2 || { echo "ERROR: HQ should have 2 devices, got $HQ_DEVICES"; exit 1; }
test -n "$HQ_SIGNATURE" || { echo "ERROR: HQ signature is empty"; exit 1; }

# Verify outpost contact.json
OUTPOST_CONTACT=$(cat alice-outpost/.fieldnotes/contact.json)
OUTPOST_DEVICES=$(echo "$OUTPOST_CONTACT" | jq '.devices | length')
OUTPOST_SIGNATURE=$(echo "$OUTPOST_CONTACT" | jq -r '.signature')

echo "Outpost devices: $OUTPOST_DEVICES"
echo "Outpost signature: ${OUTPOST_SIGNATURE:0:20}..."

test "$OUTPOST_DEVICES" -eq 2 || { echo "ERROR: Outpost should have 2 devices, got $OUTPOST_DEVICES"; exit 1; }
test -n "$OUTPOST_SIGNATURE" || { echo "ERROR: Outpost signature is empty"; exit 1; }

# Verify signatures match (both should have identical contact.json)
if [ "$HQ_SIGNATURE" != "$OUTPOST_SIGNATURE" ]; then
    echo "ERROR: Signatures do not match"
    echo "HQ: $HQ_SIGNATURE"
    echo "Outpost: $OUTPOST_SIGNATURE"
    exit 1
fi

echo "Signatures match!"

# Verify device names in contact
HQ_DEVICE1=$(echo "$HQ_CONTACT" | jq -r '.devices[0].device_name')
HQ_DEVICE2=$(echo "$HQ_CONTACT" | jq -r '.devices[1].device_name')

echo "Device 1: $HQ_DEVICE1"
echo "Device 2: $HQ_DEVICE2"

# Cleanup
rm -f /tmp/device_create_output.txt

echo ""
echo "HQ + Outpost test passed!"
