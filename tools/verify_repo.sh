#!/usr/bin/env bash
# The inventory is the only source of manifests for local and CI verification.
set -Eeuo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$repo_root"
manifest=tools/repo_manifests.txt
stage=inventory
trap 'printf "FAIL: %s [%s]\n" "$manifest" "$stage" >&2' ERR

mapfile -t manifests < tools/repo_manifests.txt
[[ ${#manifests[@]} -eq 33 ]]
LC_ALL=C sort --check --unique tools/repo_manifests.txt
for manifest in "${manifests[@]}"; do
    [[ "$manifest" != /* && "$manifest" != *..* ]]
    [[ "$manifest" == Cargo.toml || "$manifest" == */Cargo.toml ]]
    [[ -f "$manifest" && -f "${manifest%Cargo.toml}Cargo.lock" ]]
done

# Keep build artifacts outside the checkout, including when called from CI.
stage=setup
build_dir=$(mktemp -d "${TMPDIR:-/tmp}/athlesia-verify.XXXXXXXX")
trap 'rm -rf -- "$build_dir"' EXIT
export CARGO_TARGET_DIR="$build_dir/target"
export CARGO_BUILD_JOBS=1
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export RUSTFLAGS='-C debuginfo=0'
export CARGO_ENCODED_RUSTFLAGS=$'-C\x1fdebuginfo=0'
export RUSTDOCFLAGS='-C debuginfo=0'

passed=0
for manifest in "${manifests[@]}"; do
    stage=metadata
    printf '\nVERIFY: %s [%s]\n' "$manifest" "$stage"
    cargo metadata --manifest-path "$manifest" --format-version 1 --locked --no-deps > /dev/null
    stage=fmt
    printf 'VERIFY: %s [%s]\n' "$manifest" "$stage"
    cargo fmt --manifest-path "$manifest" --check
    stage=check
    printf 'VERIFY: %s [%s]\n' "$manifest" "$stage"
    cargo check --manifest-path "$manifest" --all-targets --locked -j 1
    stage=test
    printf 'VERIFY: %s [%s]\n' "$manifest" "$stage"
    cargo test --manifest-path "$manifest" --locked -j 1
    passed=$((passed + 1))
    printf 'PASS: %s (%s/%s)\n' "$manifest" "$passed" "${#manifests[@]}"
done
printf '\nRepository verification PASS: %s/%s manifests\n' "$passed" "${#manifests[@]}"
