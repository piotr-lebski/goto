#!/usr/bin/env bash
set -euo pipefail

# ── colours ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'; YELLOW='\033[0;33m'; GREEN='\033[0;32m'; RESET='\033[0m'
ok()   { echo -e "${GREEN}  ✓${RESET} $*"; }
warn() { echo -e "${YELLOW}  !${RESET} $*"; }
err()  { echo -e "${RED}  ✗${RESET} $*" >&2; exit 1; }

# ── defaults ──────────────────────────────────────────────────────────────────
VERSION=""
BIN_DIR="${HOME}/.local/bin"
SHELL_INTEGRATION=true

# ── usage ─────────────────────────────────────────────────────────────────────
usage() {
  cat <<EOF
Install goto — a bookmark-based directory navigation tool.

USAGE:
  curl -fsSL https://raw.githubusercontent.com/piotr-lebski/goto/main/install.sh | bash
  bash install.sh [FLAGS]

FLAGS:
  --version <ver>         Install a specific release (e.g. v0.2.0). Default: latest.
  --bin-dir <path>        Directory to install the binary. Default: ~/.local/bin
  --no-shell-integration  Skip adding the shell init line to your config file.
  -h, --help              Show this message.
EOF
}

# ── arg parsing ───────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      [[ $# -ge 2 ]] || err "--version requires an argument (e.g. --version v0.2.0)"
      VERSION="$2"; shift 2 ;;
    --bin-dir)
      [[ $# -ge 2 ]] || err "--bin-dir requires an argument (e.g. --bin-dir /usr/local/bin)"
      BIN_DIR="$2"; shift 2 ;;
    --no-shell-integration) SHELL_INTEGRATION=false; shift ;;
    -h|--help)              usage; exit 0 ;;
    *) err "Unknown flag: $1. Run with --help for usage." ;;
  esac
done

# ── dependency check ──────────────────────────────────────────────────────────
command -v curl &>/dev/null || err "curl is required. Install it and re-run."

# ── platform detection ────────────────────────────────────────────────────────
OS=$(uname -s)
ARCH=$(uname -m)
case "${OS}-${ARCH}" in
  Linux-x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
  Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
  *)
    err "Unsupported platform: ${OS} ${ARCH}.
See https://github.com/piotr-lebski/goto#install to build from source."
    ;;
esac

# ── version resolution ────────────────────────────────────────────────────────
if [[ -z "$VERSION" ]]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/piotr-lebski/goto/releases/latest" \
    | grep '"tag_name"' \
    | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
  [[ -n "$VERSION" ]] || err "Failed to determine latest release version."
fi

echo "Installing goto ${VERSION} for ${TARGET}…"

# ── download & verify ─────────────────────────────────────────────────────────
# Use GOTO_TMP to avoid shadowing the macOS system $TMPDIR variable.
GOTO_TMP=$(mktemp -d)
trap 'rm -rf "$GOTO_TMP"' EXIT
ARCHIVE="goto-${TARGET}-${VERSION}.tar.gz"
BASE_URL="https://github.com/piotr-lebski/goto/releases/download/${VERSION}"

curl -fsSL "${BASE_URL}/${ARCHIVE}"        -o "${GOTO_TMP}/${ARCHIVE}"
curl -fsSL "${BASE_URL}/${ARCHIVE}.sha256" -o "${GOTO_TMP}/${ARCHIVE}.sha256"

# The .sha256 file is produced by `openssl dgst -sha256 -r` (format: "<hash> *<filename>").
# Both sha256sum (Linux) and shasum -a 256 (macOS) accept this format.
cd "$GOTO_TMP"
if command -v sha256sum &>/dev/null; then
  sha256sum --check "${ARCHIVE}.sha256" --quiet
else
  shasum -a 256 -c "${ARCHIVE}.sha256" --quiet
fi
ok "Checksum verified"

# ── extract & install ─────────────────────────────────────────────────────────
tar -xzf "${GOTO_TMP}/${ARCHIVE}" -C "$GOTO_TMP"
mkdir -p "$BIN_DIR"
mv -f "${GOTO_TMP}/goto" "${BIN_DIR}/goto"
chmod +x "${BIN_DIR}/goto"
ok "Installed to ${BIN_DIR}/goto"

# ── shell integration ─────────────────────────────────────────────────────────
detect_shell() {
  case "${SHELL:-}" in
    */bash) echo "bash" ;;
    */zsh)  echo "zsh"  ;;
    */fish) echo "fish" ;;
    *)      echo "unknown" ;;
  esac
}

shell_config_file() {
  local sh="$1"
  case "$sh" in
    bash)
      if [[ "$OS" == "Darwin" ]]; then
        echo "${HOME}/.bash_profile"
      else
        echo "${HOME}/.bashrc"
      fi
      ;;
    zsh)  echo "${HOME}/.zshrc" ;;
    fish) echo "${HOME}/.config/fish/config.fish" ;;
    *)    echo "" ;;
  esac
}

DETECTED_SHELL=$(detect_shell)
CONFIG_FILE=$(shell_config_file "$DETECTED_SHELL")

if [[ -z "$CONFIG_FILE" ]]; then
  warn "Could not detect your shell (SHELL=${SHELL:-unset})."
  warn "Add the following to your shell config manually:"
  warn "  export PATH=\"${BIN_DIR}:\$PATH\""
  warn '  eval "$(goto --init)"'
else
  # Ensure fish config directory exists before any writes
  mkdir -p "$(dirname "$CONFIG_FILE")"

  # Ensure BIN_DIR is on PATH (always, idempotent)
  if ! grep -qF "${BIN_DIR}:" "$CONFIG_FILE" 2>/dev/null && \
     ! grep -qF "\"${BIN_DIR}\"" "$CONFIG_FILE" 2>/dev/null; then
    printf '\n' >> "$CONFIG_FILE"
    if [[ "$DETECTED_SHELL" == "fish" ]]; then
      echo "fish_add_path \"${BIN_DIR}\"" >> "$CONFIG_FILE"
    else
      echo "export PATH=\"${BIN_DIR}:\$PATH\"" >> "$CONFIG_FILE"
    fi
    ok "Added ${BIN_DIR} to PATH in ${CONFIG_FILE}"
  fi

  # Add shell integration (skipped by --no-shell-integration)
  if [[ "$SHELL_INTEGRATION" == "true" ]]; then
    if grep -q "goto --init" "$CONFIG_FILE" 2>/dev/null; then
      ok "Shell integration already present in ${CONFIG_FILE}"
    else
      if [[ "$DETECTED_SHELL" == "fish" ]]; then
        echo 'goto --init | source' >> "$CONFIG_FILE"
      else
        echo 'eval "$(goto --init)"' >> "$CONFIG_FILE"
      fi
      ok "Added shell integration to ${CONFIG_FILE}"
    fi
  else
    echo ""
    echo "  Shell init skipped. Add the following to ${CONFIG_FILE} manually:"
    echo '    eval "$(goto --init)"'
  fi
fi

echo ""
ok "goto ${VERSION} installed successfully!"
echo ""
if [[ -n "$CONFIG_FILE" ]]; then
  echo "  Restart your shell or run:"
  echo "    source ${CONFIG_FILE}"
else
  echo "  Add ${BIN_DIR} to your PATH and the init line to your shell config, then restart your shell."
fi
