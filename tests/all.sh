#!/bin/bash
set -e

# Run from project root

cargo install --features cli --bin footnote-cli --path .

which footnote-cli

echo "============================== Init Test        ==="
./tests/init_test.sh
sleep 5
echo "============================== Contacts Test    ==="
./tests/integration_test.sh
sleep 5
echo "============================== Device Join Test ==="
./tests/device_join_test.sh
sleep 5
echo "============================== Sync Test        ==="
./tests/sync_test.sh
sleep 5
echo "============================== Share Test       ==="
./tests/share_test.sh
echo "=== ALL CLI TESTS PASS                          ==="