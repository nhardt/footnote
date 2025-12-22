#!/bin/bash
set -e

# Cleanup and setup
rm -rf /tmp/footnotetest
mkdir -p /tmp/footnotetest
cd /tmp/footnotetest

echo "=== Step 1: Create Alice primary device (desktop) ==="
mkdir alice-desktop && cd alice-desktop
footnote-cli init --username alice --device-name desktop > /dev/null 2>&1
echo "Alice desktop created"
cd ..

echo ""
echo "=== Step 2: Create Alice secondary device (phone) ==="
cd alice-desktop
timeout 30 footnote-cli device create > /tmp/device_create_output.txt 2>&1 &
DEVICE_PID=$!
cd ..

sleep 3

CONNECTION_URL=$(grep -o 'iroh://[^[:space:]]*' /tmp/device_create_output.txt | head -1)
if [ -z "$CONNECTION_URL" ]; then
    echo "ERROR: Could not capture connection URL"
    kill $DEVICE_PID 2>/dev/null || true
    exit 1
fi

mkdir alice-phone && cd alice-phone
footnote-cli vault join phone "$CONNECTION_URL" > /dev/null 2>&1
echo "Alice phone created and joined"
cd ..

wait $DEVICE_PID 2>/dev/null || true

echo ""
echo "=== Step 3: Create Bob ==="
mkdir bob && cd bob
footnote-cli init --username bob --device-name desktop > /dev/null 2>&1
echo "Bob created"
cd ..

echo ""
echo "=== Step 4: Exchange contacts (Alice and Bob trust each other) ==="
cd alice-desktop
footnote-cli user export me > ../alice-contact.json
cd ../bob
footnote-cli trust ../alice-contact.json --petname alice
echo "Bob now trusts Alice"

footnote-cli user export me > ../bob-contact.json
cd ../alice-desktop
footnote-cli trust ../bob-contact.json --petname bob
echo "Alice now trusts Bob"
cd ..

echo ""
echo "=== Step 5: Create documents on Alice's desktop ==="
cd alice-desktop

# Document 1: Private (not shared)
UUID1="11111111-1111-1111-1111-111111111111"
cat > private.md <<ENDOFFILE
---
uuid: $UUID1
share_with: []
---

# Private Document

This is Alice's private document. It should NOT be shared with Bob.
ENDOFFILE
echo "Created private.md (not shared)"

# Document 2: Shared with Bob
UUID2="22222222-2222-2222-2222-222222222222"
cat > shared_with_bob.md <<ENDOFFILE
---
uuid: $UUID2
share_with:
  - bob
---

# Shared Document

This is Alice's document shared with Bob. Bob should receive this.
ENDOFFILE
echo "Created shared_with_bob.md (shared with bob)"
cd ..

echo ""
echo "=== Step 6: Mirror sync (Alice desktop -> Alice phone) ==="
cd alice-phone
timeout 30 footnote-cli vault listen > /tmp/mirror_listen_output.txt 2>&1 &
LISTEN_PID=$!
cd ..

sleep 2

cd alice-desktop
echo "Syncing all notes to phone..."
timeout 15 footnote-cli mirror push --device phone 2>&1 || true
cd ..

sleep 2
kill $LISTEN_PID 2>/dev/null || true

echo ""
echo "=== Step 7: Share with Bob (Alice -> Bob) ==="

# Start Bob listening for shares
cd bob
timeout 30 footnote-cli vault listen > /tmp/bob_listen_output.txt 2>&1 &
BOB_LISTEN_PID=$!
echo "Bob listening for shares (PID: $BOB_LISTEN_PID)"
cd ..

sleep 2

# Alice shares with Bob
cd alice-desktop
echo "Sharing documents with Bob..."
timeout 15 footnote-cli share bob 2>&1 || echo "Share command completed"
cd ..

sleep 2

# Stop Bob's listener
kill $BOB_LISTEN_PID 2>/dev/null || true
wait $BOB_LISTEN_PID 2>/dev/null || true

echo ""
echo "=== Step 8: Validation ==="
echo ""
echo "Checking Alice's phone (should have 2 documents + home)..."
ALICE_PHONE_COUNT=$(ls alice-phone/*.md 2>/dev/null | wc -l | xargs)
echo "Alice phone has $ALICE_PHONE_COUNT documents"

if [ ! -f alice-phone/private.md ]; then
    echo "[FAIL] private.md not found on Alice's phone"
    exit 1
fi
echo "[PASS] private.md exists on Alice's phone"

if [ ! -f alice-phone/shared_with_bob.md ]; then
    echo "[FAIL] shared_with_bob.md not found on Alice's phone"
    exit 1
fi
echo "[PASS] shared_with_bob.md exists on Alice's phone"

echo ""
echo "Checking Bob's footnotes/alice/ directory (should have 1 document)..."
if [ ! -d bob/footnotes/alice ]; then
    echo "[FAIL] bob/footnotes/alice directory does not exist"
    exit 1
fi

BOB_ALICE_COUNT=$(ls bob/footnotes/alice/*.md 2>/dev/null | wc -l | xargs)
echo "Bob has $BOB_ALICE_COUNT document(s) from Alice"

if [ "$BOB_ALICE_COUNT" -ne 1 ]; then
    echo "[FAIL] Expected 1 document from Alice, got $BOB_ALICE_COUNT"
    ls -la bob/footnotes/alice/ || true
    exit 1
fi

if [ ! -f bob/footnotes/alice/shared_with_bob.md ]; then
    echo "[FAIL] shared_with_bob.md not found in bob/footnotes/alice/"
    ls -la bob/footnotes/alice/
    exit 1
fi
echo "[PASS] shared_with_bob.md exists in bob/footnotes/alice/"

if [ -f bob/footnotes/alice/private.md ]; then
    echo "[FAIL] private.md should NOT exist in bob/footnotes/alice/"
    exit 1
fi
echo "[PASS] private.md correctly NOT shared with Bob"

# Verify content
if grep -q "This is Alice's document shared with Bob" bob/footnotes/alice/shared_with_bob.md; then
    echo "[PASS] Document content verified"
else
    echo "[FAIL] Document content mismatch"
    exit 1
fi

# Cleanup
rm -f /tmp/device_create_output.txt
rm -f /tmp/mirror_listen_output.txt

echo ""
echo "======================================"
echo "Selective sharing test PASSED!"
echo "======================================"
