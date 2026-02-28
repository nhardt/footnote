#!/bin/sh
set -e

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

source "$HOME/.cargo/env"

rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
