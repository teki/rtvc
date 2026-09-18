#!/bin/sh
# Compile every C80 tutorial example beside this script.
# Prefers `rtvc-c80` / `rtvc-tocas` on PATH, or RTVC_C80 / RTVC_TOCAS,
# otherwise `cargo run` from the repo root.

set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(CDPATH= cd -- "$HERE/../.." && pwd)
OUT="$HERE/out"
mkdir -p "$OUT"

c80() {
    if [ -n "${RTVC_C80-}" ]; then
        "$RTVC_C80" "$@"
    elif command -v rtvc-c80 >/dev/null 2>&1; then
        rtvc-c80 "$@"
    else
        cargo run --quiet --manifest-path "$ROOT/Cargo.toml" -p rtvc-c80 -- "$@"
    fi
}

tocas() {
    if [ -n "${RTVC_TOCAS-}" ]; then
        "$RTVC_TOCAS" "$@"
    elif command -v rtvc-tocas >/dev/null 2>&1; then
        rtvc-tocas "$@"
    else
        cargo run --quiet --manifest-path "$ROOT/Cargo.toml" --no-default-features --features asm-toml --bin rtvc-tocas -- "$@"
    fi
}

echo "Compiling C80 tutorial examples into $OUT"

for src in "$HERE"/*.c80; do
    stem=$(basename "$src" .c80)
    echo "  $stem"
    c80 build "$src" --origin 0x8000 \
        --emit-asm "$OUT/$stem.asm" \
        --emit-bin "$OUT/$stem.bin"
done

echo "  optimize (baseline, --no-optimize)"
c80 build "$HERE/optimize.c80" --origin 0x8000 --no-optimize \
    --emit-asm "$OUT/optimize.unopt.asm"

echo "  project/rtvc-c80.toml"
c80 build "$HERE/project/rtvc-c80.toml" \
    --emit-asm "$OUT/project.asm" \
    --emit-segments "$OUT/project.toml"

echo "  mixed/rtvc-c80.toml"
c80 build "$HERE/mixed/rtvc-c80.toml" \
    --emit-asm "$OUT/mixed.asm" \
    --emit-segments "$OUT/mixed.toml"
tocas "$OUT/mixed.toml"

echo "  tvc-usr/rtvc-c80.toml"
c80 build "$HERE/tvc-usr/rtvc-c80.toml" \
    --emit-asm "$OUT/tvc-usr.asm" \
    --emit-segments "$OUT/tvc-usr.toml"
tocas "$OUT/tvc-usr.toml"

echo "  pong/rtvc-c80.toml"
c80 build "$HERE/pong/rtvc-c80.toml" \
    --emit-asm "$OUT/pong.asm" \
    --emit-segments "$OUT/pong.toml"
tocas "$OUT/pong.toml"

echo "Done."
