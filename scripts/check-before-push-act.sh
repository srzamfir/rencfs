#!/bin/bash

set -e

export CARGO_TERM_COLOR=always
export RUSTFLAGS="-Dwarnings"
export RUSTDOCFLAGS="-Dwarnings"

cargo fmt --all
cargo clippy --release --all-targets --fix --allow-dirty --allow-staged
sudo act --action-offline-mode -W .github/workflows/build_and_tests.yaml

