#!/bin/bash
# Compile all test Java files and run them against the JVM

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

JVM_BIN="$ROOT_DIR/target/debug/jvm"
SRC_DIR="$SCRIPT_DIR/src"
OUT_DIR="$SCRIPT_DIR/out"
TIMEOUT=30

# Build JVM first
echo "Building JVM..."
cargo build --manifest-path "$ROOT_DIR/Cargo.toml" 2>/dev/null

if [ ! -f "$JVM_BIN" ]; then
    echo "ERROR: JVM binary not found at $JVM_BIN"
    exit 1
fi

# Compile Java sources
mkdir -p "$OUT_DIR"
echo "Compiling Java files..."
javac -d "$OUT_DIR" "$SRC_DIR"/*.java

PASS=0
FAIL=0
TOTAL=0

run_with_timeout() {
    local secs="$1"
    shift
    "$@" &
    local pid=$!
    (sleep "$secs" && kill -9 "$pid" 2>/dev/null) &
    local killer=$!
    wait "$pid" 2>/dev/null
    local ret=$?
    kill "$killer" 2>/dev/null
    wait "$killer" 2>/dev/null
    return $ret
}

echo ""
echo "Running tests (timeout: ${TIMEOUT}s per test)..."
echo ""

for java_file in "$SRC_DIR"/*.java; do
    class=$(basename "$java_file" .java)

    if [ ! -f "$OUT_DIR/${class}.class" ]; then
        continue
    fi

    TOTAL=$((TOTAL + 1))
    printf "  %-25s ... " "$class"

    if run_with_timeout "$TIMEOUT" "$JVM_BIN" --cp "$OUT_DIR" "$class" >/dev/null 2>&1; then
        echo "PASS"
        PASS=$((PASS + 1))
    else
        echo "FAIL"
        FAIL=$((FAIL + 1))
    fi
done

echo ""
echo "$PASS/$TOTAL passed, $FAIL failed"
