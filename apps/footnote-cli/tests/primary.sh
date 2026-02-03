#!/bin/bash
set -e


echo "// Setup"

rm -rf /tmp/footnotetest
mkdir -p /tmp/footnotetest
cd /tmp/footnotetest

echo "// Init Primary"
mkdir alice-vault-desktop && cd alice-vault-desktop
footnote-cli vault create-primary alice alice-desktop

test -d .footnote || { echo "ERROR: .footnote not found"; exit 1; }
test -f .footnote/id_key || { echo "ERROR: id_key not found"; exit 1; }
test -f .footnote/device_key || { echo "ERROR: device_key not found"; exit 1; }

cd ..

echo "// Init Standalone (pre-join state)"
mkdir alice-vault-laptop && cd alice-vault-laptop
footnote-cli vault create-standalone
test -d .footnote || { echo "ERROR: .footnote not found"; exit 1; }
test ! -f .footnote/id_key || { echo "ERROR: id_key should only be on primary"; exit 1; }
test ! -f .footnote/device_key || { echo "ERROR: device_key should not exist until join"; exit 1; }


echo "// Init test passed!"
