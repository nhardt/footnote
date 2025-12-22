#!/bin/bash
set -e

# Run from project root

cargo install --features cli --bin footnote-cli --path .

which footnote-cli

./tests/init_test.sh
./tests/device_join_test.sh
./tests/mirror_sync_test.sh
./tests/selective_sharing_test.sh
./tests/integration_test.sh