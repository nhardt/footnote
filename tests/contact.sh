#!/bin/bash
set -e  # Exit on any error

# Cleanup and setup
rm -rf /tmp/footnotetest
mkdir -p /tmp/footnotetest
cd /tmp/footnotetest

# Alice desktop device
mkdir alice-desktop && cd alice-desktop
footnote-cli vault create-primary ali desktop
cd ..

# Bob primary device
mkdir bob-desktop && cd bob-desktop
footnote-cli vault create-primary bob bob-desktop
cd ..

# Test export/import (offline contact sharing)
cd alice-desktop
footnote-cli contact export > ../alice-contact.json
cd ../bob-desktop
footnote-cli contact import alice-from-work ../alice-contact.json
footnote-cli contact read | grep alice-from-work
footnote-cli contact read | grep ali

echo "Contact import/export passed!"
