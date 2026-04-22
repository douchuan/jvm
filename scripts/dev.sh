#!/bin/bash
# dev.sh — convenience script for JVM development
#
# Usage:
#   ./scripts/dev.sh build              # Build workspace (debug)
#   ./scripts/dev.sh build-release      # Build workspace (release)
#   ./scripts/dev.sh run <Class> [args] # Run a Java class
#   ./scripts/dev.sh test               # Run all tests
#   ./scripts/dev.sh javap <file>       # Disassemble a class file
#   ./scripts/dev.sh clean              # Clean build artifacts
set -euo pipefail

case "${1:-help}" in
  build)
    cargo build --workspace
    ;;
  build-release)
    cargo build --workspace --release
    ;;
  run)
    cargo run -p jvm -- "${@:2}"
    ;;
  test)
    cargo test --workspace
    ;;
  javap)
    cargo run -p javap -- "${@:2}"
    ;;
  clean)
    cargo clean
    ;;
  help|*)
    echo "Usage: ./scripts/dev.sh <command> [args...]"
    echo ""
    echo "Commands:"
    echo "  build              Build workspace (debug)"
    echo "  build-release      Build workspace (release)"
    echo "  run <Class> [args] Run a Java class"
    echo "    --classpath <p>  Set classpath"
    echo "  test               Run all tests"
    echo "  javap <classfile>  Disassemble a class file"
    echo "  clean              Clean build artifacts"
    echo "  help               Show this message"
    ;;
esac
