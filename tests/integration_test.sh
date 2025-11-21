#!/bin/bash
set -e  # Exit on any error

# Cleanup and setup
rm -rf /tmp/footnotetest
mkdir -p /tmp/footnotetest
cd /tmp/footnotetest

# Alice primary device
mkdir alice-primary && cd alice-primary
footnote init --device-name alice-desktop
footnote user read
cd ..

# Bob primary device
mkdir bob-primary && cd bob-primary
footnote init --device-name bob-desktop
footnote user read
cd ..

# Test export/import (offline contact sharing)
cd alice-primary
footnote user export me > ../alice-contact.yaml
cd ../bob-primary
footnote trust ../alice-contact.yaml --petname alice
footnote user read  # Should show alice in trusted users

echo "Basic test passed!"
