#!/bin/sh
set -e

# gx installer
# Usage: curl -fsSL https://raw.githubusercontent.com/joshuaboys/gx/main/install.sh | sh

REPO="joshuaboys/gx"
INSTALL_DIR="$HOME/.local/bin"
GX_BIN="$INSTALL_DIR/gx"

# --- helpers ---

info() { printf '\033[1;34m==>\033[0m %s\n' "$1"; }
warn() { printf '\033[1;33mwarning:\033[0m %s\n' "$1"; }
error() { printf '\033[1;31merror:\033[0m %s\n' "$1" >&2; exit 1; }

command_exists() { command -v "$1" >/dev/null 2>&1; }

download() {
  local url="$1" dest="$2"
  if command_exists curl; then
    curl -fsSL "$url" -o "$dest"
  elif command_exists wget; then
    wget -qO "$dest" "$url"
  else
    error "Need curl or wget to download gx"
  fi
}

sha256_file() {
  if command_exists sha256sum; then
    sha256sum "$1" | awk '{print $1}'
  elif command_exists shasum; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    error "Need sha256sum or shasum to verify release checksums"
  fi
}

verify_checksum() {
  local asset="$1" file="$2" sums="$3" expected actual
  expected=$(awk -v asset="$asset" '$2 == asset || $2 == "*" asset {print $1; exit}' "$sums")
  if [ -z "$expected" ]; then
    error "Checksum for ${asset} not found in SHA256SUMS"
  fi

  actual=$(sha256_file "$file")
  if [ "$actual" != "$expected" ]; then
    error "Checksum verification failed for ${asset}"
  fi
}

detect_os() {
  case "$(uname -s)" in
    Linux*)  echo "linux" ;;
    Darwin*) echo "darwin" ;;
    *)       error "Unsupported OS: $(uname -s)" ;;
  esac
}

detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64)  echo "x64" ;;
    aarch64|arm64) echo "aarch64" ;;
    *)             error "Unsupported architecture: $(uname -m)" ;;
  esac
}

detect_shell() {
  basename "${SHELL:-/bin/sh}"
}

shell_rc_file() {
  case "$1" in
    zsh)  echo "$HOME/.zshrc" ;;
    bash)
      case "$(uname -s)" in
        Darwin*)
          if [ -f "$HOME/.bash_profile" ]; then
            echo "$HOME/.bash_profile"
          elif [ -f "$HOME/.profile" ]; then
            echo "$HOME/.profile"
          else
            echo "$HOME/.bashrc"
          fi
          ;;
        *) echo "$HOME/.bashrc" ;;
      esac
      ;;
    fish) echo "$HOME/.config/fish/conf.d/gx.fish" ;;
    *)    echo "" ;;
  esac
}

# --- install strategy ---

# Temp dir for the download, cleaned up on exit. Script-scoped (not `local`)
# so the EXIT trap still sees it after install_prebuilt returns.
GX_TMPDIR=""
cleanup() { [ -n "$GX_TMPDIR" ] && rm -rf "$GX_TMPDIR"; }
trap cleanup EXIT

install_prebuilt() {
  local os="$1" arch="$2"
  local asset="gx-${os}-${arch}"
  local base="https://github.com/${REPO}/releases/latest/download"

  if ! command_exists curl && ! command_exists wget; then
    error "Need curl or wget to install gx"
  fi

  info "Downloading prebuilt binary ($os-$arch)..."
  GX_TMPDIR=$(mktemp -d)
  local bin="$GX_TMPDIR/$asset"
  local sums="$GX_TMPDIR/SHA256SUMS"

  # Download directly; a 404 (no asset for this target) makes curl -f / wget
  # exit non-zero, which we turn into a clear error (no source fallback).
  if ! download "$base/$asset" "$bin" 2>/dev/null; then
    error "No prebuilt binary for ${os}-${arch} in the latest release.
Supported targets: linux-x64, linux-aarch64, darwin-x64, darwin-aarch64.
Download manually from https://github.com/${REPO}/releases, or build from
source with a Rust toolchain: 'cargo build --release -p gx'."
  fi
  download "$base/SHA256SUMS" "$sums"
  verify_checksum "$asset" "$bin" "$sums"
  info "Verified SHA-256 checksum"

  mkdir -p "$INSTALL_DIR"
  mv "$bin" "$GX_BIN"
  chmod +x "$GX_BIN"
}

# --- shell integration ---

setup_shell() {
  local shell rc_file init_line marker
  shell=$(detect_shell)
  rc_file=$(shell_rc_file "$shell")

  if [ -z "$rc_file" ]; then
    warn "Unrecognised shell ($shell). Add shell integration manually:"
    echo '  eval "$(gx shell-init)"'
    return
  fi

  # Fish uses different syntax
  if [ "$shell" = "fish" ]; then
    init_line='gx shell-init | source'
  else
    init_line='eval "$(gx shell-init)"'
  fi

  marker="# gx"

  # Already configured?
  if [ -f "$rc_file" ] && grep -qF "$marker" "$rc_file" 2>/dev/null; then
    info "Shell integration already in $rc_file"
    return
  fi

  # Create parent dirs for fish
  mkdir -p "$(dirname "$rc_file")"

  printf '\n%s\n%s\n' "$marker" "$init_line" >> "$rc_file"
  info "Added shell integration to $rc_file"
}

# --- PATH check ---

ensure_path() {
  case ":$PATH:" in
    *":$INSTALL_DIR:"*) return ;;
  esac

  warn "$INSTALL_DIR is not on your PATH"

  local shell rc_file
  shell=$(detect_shell)
  rc_file=$(shell_rc_file "$shell")

  if [ -n "$rc_file" ] && [ -f "$rc_file" ]; then
    if ! grep -qF "$INSTALL_DIR" "$rc_file" 2>/dev/null; then
      if [ "$shell" = "fish" ]; then
        printf '\nfish_add_path %s\n' "$INSTALL_DIR" >> "$rc_file"
      else
        printf '\nexport PATH="%s:$PATH"\n' "$INSTALL_DIR" >> "$rc_file"
      fi
      info "Added $INSTALL_DIR to PATH in $rc_file"
    fi
  else
    echo "  Add this to your shell config:"
    echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
  fi
}

# --- main ---

main() {
  info "Installing gx..."

  local os arch
  os=$(detect_os)
  arch=$(detect_arch)

  install_prebuilt "$os" "$arch"

  setup_shell
  ensure_path

  echo ""
  info "gx installed to $GX_BIN"
  info "Restart your shell or run: exec \$SHELL"
  echo ""
}

main
