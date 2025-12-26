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
footnote-cli contact export > ../alice-contact.json
cd ..

# Create Bob primary device
mkdir bob-desktop && cd bob-desktop
echo "Creating Bob primary device..."
footnote-cli vault create-primary bob bob-desktop
footnote-cli contact export > ../bob-contact.json
footnote-cli contact import alice ../alice-contact.json

cd ../alice-desktop
footnote-cli contact import bob ../bob-contact.json
echo "Alice may now share with bob"

echo "Create documents on Alice's desktop"
footnote-cli note create surprise_party.md "party party party"
footnote-cli note create shared_with_bob.md "a story to share with bob"
footnote-cli note update shared_with_bob.md "a story to share with bob" --share bob 

echo "Share with Bob (Alice -> Bob) ==="
cd ../bob-desktop
timeout 30 footnote-cli service share-listen > /tmp/bob_listen_output.txt 2>&1 &
BOB_LISTEN_PID=$!
echo "Bob listening for shares (PID: $BOB_LISTEN_PID)"

sleep 10

# Alice shares with Bob
cd ../alice-desktop
echo "Sharing documents with Bob..."
timeout 15 footnote-cli service share bob 2>&1 || echo "Share command completed"
cd ..

# Stop Bob's listener
kill $BOB_LISTEN_PID 2>/dev/null || true
wait $BOB_LISTEN_PID 2>/dev/null || true

echo ""
echo "Checking Bob's footnotes/alice/ directory (should have 1 document)..."
if [ ! -d bob-desktop/footnotes/alice ]; then
    echo "[FAIL] bob/footnotes/alice directory does not exist"
    exit 1
fi

BOB_ALICE_COUNT=$(ls bob-desktop/footnotes/alice/*.md 2>/dev/null | wc -l | xargs)
echo "Bob has $BOB_ALICE_COUNT document(s) from Alice"

if [ "$BOB_ALICE_COUNT" -ne 1 ]; then
    echo "[FAIL] Expected 1 document from Alice, got $BOB_ALICE_COUNT"
    ls -la bob-desktop/footnotes/alice/ || true
    exit 1
fi

if [ ! -f bob-desktop/footnotes/alice/shared_with_bob.md ]; then
    echo "[FAIL] shared_with_bob.md not found in bob/footnotes/alice/"
    ls -la bob-desktop/footnotes/alice/
    exit 1
fi
echo "[PASS] shared_with_bob.md exists in bob-desktop/footnotes/alice/"

if [ -f bob-desktop/footnotes/alice/private.md ]; then
    echo "[FAIL] private.md should NOT exist in bob-desktop/footnotes/alice/"
    exit 1
fi
echo "[PASS] private.md correctly NOT shared with Bob"

# Verify content
if grep -q "a story to share with bob" bob-desktop/footnotes/alice/shared_with_bob.md; then
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
