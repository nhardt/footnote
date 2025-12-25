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
echo "Starting device authorization..."
timeout 30 footnote-cli service join-listen > /tmp/device_create_output.txt 2>&1 &
JOIN_LISTEN_PID=$!
cd ..

# Wait for connection URL to appear
echo "Waiting for connection URL..."
sleep 3
echo "Extracting connection url from "
cat /tmp/device_create_output.txt

# Extract connection URL from output
CONNECTION_URL=$(cat /tmp/device_create_output.txt | jq -r '.join_url')

if [ -z "$CONNECTION_URL" ]; then
    echo "ERROR: Could not capture connection URL"
    cat /tmp/device_create_output.txt
    kill $DEVICE_PID 2>/dev/null || true
    exit 1
fi

echo "Connection URL: $CONNECTION_URL"

# Create Alice secondary device and join
mkdir alice-laptop && cd alice-laptop
echo "Creating Alice secondary device..."
footnote-cli vault create-secondary alice-phone
footnote-cli service join "$CONNECTION_URL"
cd ..

wait $JOIN_LISTEN_PID 2>/dev/null || true

echo "Pairing complete"
echo ""
echo "Creating note on primary"
cd alice-desktop

# Create a test note with UUID
TEST_UUID="550e8400-e29b-41d4-a716-446655440001"
cat > test_note.md <<ENDOFFILE
---
uuid: $TEST_UUID
share_with: []
---

# Test Note

This is a test note created on primary device.
It should sync to the secondary device.
ENDOFFILE

echo "Created notes/test_note.md with UUID: $TEST_UUID"
cd ..

echo ""
echo "Sync primary to secondary"

# Start mirror listener on phone in background
cd alice-phone
timeout 30 footnote-cli service replicate-listen > /tmp/replicate_listen_output.txt 2>&1 &
LISTEN_PID=$!
echo "Phone listening for sync (PID: $LISTEN_PID)"
cd ..

# Wait for listener to start
sleep 2

# Push from primary
cd alice-primary
echo "Pushing from primary to phone..."
footnote-cli service replicate alice-phone
cd ..

# Give sync time to complete
sleep 2

# Stop listener
kill $LISTEN_PID 2>/dev/null || true
wait $LISTEN_PID 2>/dev/null || true

echo ""
echo "=== Step 5: Verify sync ==="

# Check if note exists on phone
if [ -f alice-phone/test_note.md ]; then
    echo "Note synced successfully to phone"

    # Verify UUID matches
    PHONE_UUID=$(grep "uuid:" alice-phone/test_note.md | awk '{print $2}')
    if [ "$PHONE_UUID" = "$TEST_UUID" ]; then
        echo "UUID matches: $PHONE_UUID"
    else
        echo "UUID mismatch!"
        echo "  Expected: $TEST_UUID"
        echo "  Got: $PHONE_UUID"
        exit 1
    fi

    # Verify content
    if grep -q "This is a test note created on primary device" alice-phone/test_note.md; then
        echo "Note content verified"
    else
        echo "Note content mismatch!"
        cat alice-phone/test_note.md
        exit 1
    fi
else
    echo "Note not found on phone!"
    echo "Contents of alice-phone/:"
    ls -la alice-phone/ || echo "Directory does not exist"
    exit 1
fi

# Cleanup
rm -f /tmp/device_create_output.txt
rm -f /tmp/mirror_listen_output.txt

echo ""
echo "Mirror sync test passed!"
