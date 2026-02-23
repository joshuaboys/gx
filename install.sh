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
    bash) echo "$HOME/.bashrc" ;;
    fish) echo "$HOME/.config/fish/conf.d/gx.fish" ;;
    *)    echo "" ;;
  esac
}

# --- install strategies ---

try_prebuilt() {
  local os="$1" arch="$2"
  local asset="gx-${os}-${arch}"
  local url="https://github.com/${REPO}/releases/latest/download/${asset}"

  info "Checking for prebuilt binary ($os-$arch)..."
  local http_code
  if command_exists curl; then
    http_code=$(curl -sL -o /dev/null -w "%{http_code}" "$url" 2>/dev/null) || http_code="000"
  elif command_exists wget; then
    http_code=$(wget --spider -S "$url" 2>&1 | grep "HTTP/" | tail -1 | awk '{print $2}') || http_code="000"
  else
    return 1
  fi

  if [ "$http_code" != "200" ]; then
    info "No prebuilt binary available, building from source..."
    return 1
  fi

  info "Downloading prebuilt binary..."
  mkdir -p "$INSTALL_DIR"
  if command_exists curl; then
    curl -fsSL "$url" -o "$GX_BIN"
  else
    wget -qO "$GX_BIN" "$url"
  fi
  chmod +x "$GX_BIN"
  return 0
}

build_from_source() {
  # Ensure bun is available
  if ! command_exists bun; then
    info "Bun not found, installing..."
    if command_exists curl; then
      curl -fsSL https://bun.sh/install | bash
    elif command_exists wget; then
      wget -qO- https://bun.sh/install | bash
    else
      error "Need curl or wget to install Bun"
    fi
    # Source bun into current session
    export BUN_INSTALL="$HOME/.bun"
    export PATH="$BUN_INSTALL/bin:$PATH"
    if ! command_exists bun; then
      error "Bun installation failed"
    fi
  fi

  local tmpdir
  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT

  info "Cloning gx..."
  if command_exists git; then
    git clone --depth 1 "https://github.com/${REPO}.git" "$tmpdir/gx" 2>/dev/null
  else
    error "Git is required to build from source"
  fi

  info "Building..."
  cd "$tmpdir/gx"
  bun install --frozen-lockfile 2>/dev/null || bun install
  bun run build

  mkdir -p "$INSTALL_DIR"
  cp gx "$GX_BIN"
  chmod +x "$GX_BIN"
  cd - >/dev/null
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

  if ! try_prebuilt "$os" "$arch"; then
    build_from_source
  fi

  ensure_path
  setup_shell

  echo ""
  info "gx installed to $GX_BIN"
  info "Restart your shell or run: exec \$SHELL"
  echo ""
}

main
