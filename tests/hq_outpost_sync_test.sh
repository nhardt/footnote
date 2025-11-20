#!/bin/bash
set -e

# Cleanup and setup
rm -rf /tmp/fieldnotest
mkdir -p /tmp/fieldnotest
cd /tmp/fieldnotest

echo "=== Step 1: Create Alice HQ ==="
mkdir alice-hq && cd alice-hq
fieldnote hq create --username alice --device-name desktop > /dev/null 2>&1
echo "HQ created"
cd ..

echo ""
echo "=== Step 2: Create Alice outpost ==="
# Start device authorization in background
cd alice-hq
timeout 30 fieldnote device create > /tmp/device_create_output.txt 2>&1 &
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

# Create outpost and join
mkdir alice-outpost && cd alice-outpost
echo "Attempting to join with: $CONNECTION_URL"
if ! fieldnote device create remote "$CONNECTION_URL" --device-name phone 2>&1; then
    echo "ERROR: Failed to create remote device"
    echo "Mirror listen output:"
    cat /tmp/device_create_output.txt || true
    cd ..
    kill $DEVICE_PID 2>/dev/null || true
    exit 1
fi
echo "Outpost created and joined"
cd ..

# Wait for authorization to complete
wait $DEVICE_PID 2>/dev/null || true

echo ""
echo "=== Step 3: Add note on HQ ==="
cd alice-hq

# Create a test note with UUID
TEST_UUID="550e8400-e29b-41d4-a716-446655440001"
cat > notes/test_note.md <<EOF
---
uuid: $TEST_UUID
share_with: []
---

# Test Note

This is a test note created on HQ.
It should sync to the outpost device.
EOF

echo "Created notes/test_note.md with UUID: $TEST_UUID"
cd ..

echo ""
echo "=== Step 4: Sync HQ to outpost ==="

# Start mirror listener on outpost in background
cd alice-outpost
timeout 30 fieldnote mirror listen > /tmp/mirror_listen_output.txt 2>&1 &
LISTEN_PID=$!
echo "Outpost listening for sync (PID: $LISTEN_PID)"
cd ..

# Wait for listener to start
sleep 2

# Push from HQ
cd alice-hq
echo "Pushing from HQ to outpost..."
timeout 15 fieldnote mirror push --device phone 2>&1 || echo "Push command completed"
cd ..

# Give sync time to complete
sleep 2

# Stop listener
kill $LISTEN_PID 2>/dev/null || true
wait $LISTEN_PID 2>/dev/null || true

echo ""
echo "=== Step 5: Verify sync ==="

# Check if note exists on outpost
if [ -f alice-outpost/notes/test_note.md ]; then
    echo "✓ Note synced successfully to outpost"

    # Verify UUID matches
    OUTPOST_UUID=$(grep "uuid:" alice-outpost/notes/test_note.md | awk '{print $2}')
    if [ "$OUTPOST_UUID" = "$TEST_UUID" ]; then
        echo "✓ UUID matches: $OUTPOST_UUID"
    else
        echo "✗ UUID mismatch!"
        echo "  Expected: $TEST_UUID"
        echo "  Got: $OUTPOST_UUID"
        exit 1
    fi

    # Verify content
    if grep -q "This is a test note created on HQ" alice-outpost/notes/test_note.md; then
        echo "✓ Note content verified"
    else
        echo "✗ Note content mismatch!"
        cat alice-outpost/notes/test_note.md
        exit 1
    fi
else
    echo "✗ Note not found on outpost!"
    echo "Contents of alice-outpost/notes/:"
    ls -la alice-outpost/notes/ || echo "Directory does not exist"
    exit 1
fi

# Cleanup
rm -f /tmp/device_create_output.txt
rm -f /tmp/mirror_listen_output.txt

echo ""
echo "✓ HQ + Outpost sync test passed!"
