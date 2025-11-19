#!/bin/bash
set -e  # Exit on any error

# Cleanup and setup
rm -rf /tmp/fieldnotest
mkdir -p /tmp/fieldnotest
cd /tmp/fieldnotest

# Alice HQ
mkdir alice-hq && cd alice-hq
fieldnote hq create --device-name alice-desktop
fieldnote user read
cd ..

# Bob HQ
mkdir bob-hq && cd bob-hq
fieldnote hq create --device-name bob-desktop
fieldnote user read
cd ..

# Test export/import (offline contact sharing)
cd alice-hq
fieldnote user export me > ../alice-contact.yaml
cd ../bob-hq
fieldnote user import ../alice-contact.yaml --petname alice
fieldnote user read  # Should show alice in embassies

echo "Basic test passed!"
