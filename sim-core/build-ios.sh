#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "=== Building Planet Architect sim-core for iOS ==="

echo ""
echo "[1/2] Building for iOS Simulator (arm64)..."
cargo build --release --target aarch64-apple-ios-sim

echo ""
echo "[2/2] Building for iOS Device (arm64)..."
cargo build --release --target aarch64-apple-ios

echo ""
echo "=== Build complete ==="
echo "  Simulator: target/aarch64-apple-ios-sim/release/libplanet_architect_sim.a"
echo "  Device:    target/aarch64-apple-ios/release/libplanet_architect_sim.a"
