#!/bin/bash
set -e

# Cleanup and setup
rm -rf /tmp/footnotetest
mkdir -p /tmp/footnotetest
cd /tmp/footnotetest

echo "=== Step 1: Create Alice primary device ==="
mkdir alice-primary && cd alice-primary
footnote init --username alice --device-name desktop > /dev/null 2>&1
echo "Primary device created"
cd ..

echo ""
echo "=== Step 2: Create Alice secondary device ==="
# Start device authorization in background
cd alice-primary
timeout 30 footnote device create > /tmp/device_create_output.txt 2>&1 &
DEVICE_PID=$!
cd ..

# Wait for connection URL and endpoint initialization
sleep 5

# Extract connection URL
CONNECTION_URL=$(grep -o 'iroh://[^[:space:]]*' /tmp/device_create_output.txt | head -1)

if [ -z "$CONNECTION_URL" ]; then
    echo "ERROR: Could not capture connection URL"
    cat /tmp/device_create_output.txt
    kill $DEVICE_PID 2>/dev/null || true
    exit 1
fi

echo "Connection URL captured"

# Create secondary device and join
mkdir alice-phone && cd alice-phone
echo "Attempting to join with: $CONNECTION_URL"
if ! footnote mirror from "$CONNECTION_URL" --device-name phone 2>&1; then
    echo "ERROR: Failed to create remote device"
    echo "Mirror listen output:"
    cat /tmp/device_create_output.txt || true
    cd ..
    kill $DEVICE_PID 2>/dev/null || true
    exit 1
fi
echo "Secondary device created and joined"
cd ..

# Wait for authorization to complete
wait $DEVICE_PID 2>/dev/null || true

echo ""
echo "=== Step 3: Add note on primary device ==="
cd alice-primary

# Create a test note with UUID
TEST_UUID="550e8400-e29b-41d4-a716-446655440001"
cat > notes/test_note.md <<ENDOFFILE
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
echo "=== Step 4: Sync primary to phone ==="

# Start mirror listener on phone in background
cd alice-phone
timeout 30 footnote mirror listen > /tmp/mirror_listen_output.txt 2>&1 &
LISTEN_PID=$!
echo "Phone listening for sync (PID: $LISTEN_PID)"
cd ..

# Wait for listener to start
sleep 2

# Push from primary
cd alice-primary
echo "Pushing from primary to phone..."
timeout 15 footnote mirror push --device phone 2>&1 || echo "Push command completed"
cd ..

# Give sync time to complete
sleep 2

# Stop listener
kill $LISTEN_PID 2>/dev/null || true
wait $LISTEN_PID 2>/dev/null || true

echo ""
echo "=== Step 5: Verify sync ==="

# Check if note exists on phone
if [ -f alice-phone/notes/test_note.md ]; then
    echo "✓ Note synced successfully to phone"

    # Verify UUID matches
    PHONE_UUID=$(grep "uuid:" alice-phone/notes/test_note.md | awk '{print $2}')
    if [ "$PHONE_UUID" = "$TEST_UUID" ]; then
        echo "✓ UUID matches: $PHONE_UUID"
    else
        echo "✗ UUID mismatch!"
        echo "  Expected: $TEST_UUID"
        echo "  Got: $PHONE_UUID"
        exit 1
    fi

    # Verify content
    if grep -q "This is a test note created on primary device" alice-phone/notes/test_note.md; then
        echo "✓ Note content verified"
    else
        echo "✗ Note content mismatch!"
        cat alice-phone/notes/test_note.md
        exit 1
    fi
else
    echo "✗ Note not found on phone!"
    echo "Contents of alice-phone/notes/:"
    ls -la alice-phone/notes/ || echo "Directory does not exist"
    exit 1
fi

# Cleanup
rm -f /tmp/device_create_output.txt
rm -f /tmp/mirror_listen_output.txt

echo ""
echo "✓ Mirror sync test passed!"
