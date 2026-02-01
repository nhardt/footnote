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
mkdir alice-laptop && cd alice-laptop
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
footnote-cli service join "$CONNECTION_URL" alice-laptop
cd ..

wait $JOIN_LISTEN_PID 2>/dev/null || true

echo "Pairing complete"
echo ""
echo "Creating note on primary"
cd alice-desktop

NOTE_CONTENT=`uuidgen`

footnote-cli note create created_on_primary.md $NOTE_CONTENT
cd ..

echo ""
echo "Sync primary to secondary"

cd alice-laptop
timeout 30 footnote-cli service replicate-listen > /tmp/replicate_listen_output.txt 2>&1 &
LISTEN_PID=$!
echo "Laptop listening for sync (PID: $LISTEN_PID)"
cd ..

# Wait for listener to start
sleep 2

# Push from primary
cd alice-desktop
echo "Pushing from desktop to laptop..."
footnote-cli service replicate alice-laptop
cd ..

# Give sync time to complete
sleep 2

# Stop listener
kill $LISTEN_PID 2>/dev/null || true
wait $LISTEN_PID 2>/dev/null || true

echo ""
echo "=== Verify replication ==="

# Check if note exists on laptop
if [ -f alice-laptop/created_on_primary.md ]; then
    echo "Note synced successfully to laptop"

    if grep -q "$NOTE_CONTENT" alice-laptop/created_on_primary.md; then
        echo "Note content verified"
    else
        echo "Note content mismatch!"
        cat alice-laptop/created_on_primary.md
        exit 1
    fi
else
    echo "Note not found on laptop!"
    echo "Contents of alice-laptop/:"
    ls -la alice-laptop/ || echo "Directory does not exist"
    exit 1
fi

# Cleanup
rm -f /tmp/join_listen_output.txt
rm -f /tmp/replicate_listen_output.txt

echo ""
echo "Mirror sync test passed!"
