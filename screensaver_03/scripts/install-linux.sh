#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
PROJECT_DIR=$(cd -- "$SCRIPT_DIR/.." && pwd)
PREFIX="${PREFIX:-${HOME}/.local}"
DESTDIR="${DESTDIR:-}"
INSTALL_ROOT="${DESTDIR}${PREFIX}"
BIN_DIR="$INSTALL_ROOT/bin"
APP_DIR="$INSTALL_ROOT/share/applications"
RUNTIME_BIN_DIR="$PREFIX/bin"

mkdir -p "$BIN_DIR" "$APP_DIR"

echo "==> Building solverforge-screensaver"
cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"

echo "==> Installing binary"
install -m 0755 "$PROJECT_DIR/target/release/solverforge-screensaver" "$BIN_DIR/solverforge-screensaver"
install -m 0755 "$PROJECT_DIR/scripts/solverforge-screensaver-launch" "$BIN_DIR/solverforge-screensaver-launch"
install -m 0755 "$PROJECT_DIR/scripts/solverforge-screensaverctl" "$BIN_DIR/solverforge-screensaverctl"

cat > "$APP_DIR/solverforge-screensaver.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=SolverForge Screensaver
Comment=Silent phosphor screensaver in Rust
Exec=${RUNTIME_BIN_DIR}/solverforge-screensaver-launch
Terminal=false
Categories=Graphics;
Keywords=screensaver;solverforge;rust;
StartupNotify=false
DESKTOP

echo ""
echo "Installed to:"
echo "  ${BIN_DIR}/solverforge-screensaver"
echo "  ${BIN_DIR}/solverforge-screensaver-launch"
echo "  ${BIN_DIR}/solverforge-screensaverctl"
echo ""
echo "Run now:"
echo "  ${RUNTIME_BIN_DIR}/solverforge-screensaverctl run"
echo ""
echo "Set as idle screensaver on Linux:"
echo "  ${RUNTIME_BIN_DIR}/solverforge-screensaverctl set --timeout 300"
