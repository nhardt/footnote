#!/bin/bash
set -e

# Run from project root

cargo install --force --bin footnote-cli --path .

which footnote-cli

echo "============================== Primary Test     ==="
./tests/primary.sh
echo "============================== Secondary Test   ==="
./tests/secondary.sh
echo "============================== Sync Test        ==="
./tests/replicate.sh
echo "============================== Contact Test     ==="
./tests/contact.sh
echo "============================== Share Test       ==="
./tests/share.sh
echo "=== ALL CLI TESTS PASS                          ==="
