#!/bin/bash
set -e

rm -rf /tmp/footnotetest
mkdir -p /tmp/footnotetest
cd /tmp/footnotetest

mkdir alice-vault && cd alice-vault
footnote-cli vault create-primary alice-desktop

test -d .footnote || { echo "ERROR: .footnotes not found"; exit 1; }
test -f .footnote/id_key || { echo "ERROR: id_key not found"; exit 1; }
test -f .footnote/device_key || { echo "ERROR: device_key found"; exit 1; }

echo "Init test passed!"
