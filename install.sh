#!/usr/bin/env bash
set -euo pipefail

MODE="install"
WITH_SYSTEMD=0
PREFIX="/usr/local"
APP_DIR="/etc/arma"
BIN_NAME="arma"
SERVICE_USER="arma"
SERVICE_GROUP="arma"
BINARY_URL=""
REPO="parkjangwon/arma"
TAG=""
DRY_RUN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --with-systemd)
      WITH_SYSTEMD=1
      shift
      ;;
    --prefix)
      PREFIX="$2"
      shift 2
      ;;
    --app-dir)
      APP_DIR="$2"
      shift 2
      ;;
    --service-user)
      SERVICE_USER="$2"
      SERVICE_GROUP="$2"
      shift 2
      ;;
    --binary-url)
      BINARY_URL="$2"
      shift 2
      ;;
    --repo)
      REPO="$2"
      shift 2
      ;;
    --tag)
      TAG="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    uninstall)
      MODE="uninstall"
      shift
      ;;
    install)
      MODE="install"
      shift
      ;;
    *)
      echo "Unknown argument: $1"
      echo "Usage: ./install.sh [install|uninstall] [--with-systemd] [--prefix PATH] [--app-dir PATH] [--service-user USER] [--binary-url URL] [--repo OWNER/REPO] [--tag TAG] [--dry-run]"
      exit 1
      ;;
  esac
done

LIB_DIR="$PREFIX/lib/$BIN_NAME"
BIN_DIR="$PREFIX/bin"
TARGET_BIN="$LIB_DIR/$BIN_NAME"
WRAPPER_BIN="$BIN_DIR/$BIN_NAME"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing required command: $1"
    exit 1
  }
}

resolve_latest_tag() {
  local latest parsed
  latest="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest")"
  parsed="$(printf '%s' "$latest" | tr -d '\n' | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
  if [[ -z "$parsed" ]]; then
    echo "Failed to resolve latest release tag for $REPO. Use --tag or --binary-url."
    exit 1
  fi
  echo "$parsed"
}

detect_asset_name() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"

  case "$os" in
    linux)
      case "$arch" in
        x86_64|amd64) echo "arma-linux-amd64" ;;
        i386|i686) echo "arma-linux-386" ;;
        *) echo "Unsupported Linux architecture: $arch"; exit 1 ;;
      esac
      ;;
    darwin)
      case "$arch" in
        x86_64) echo "arma-macos-amd64" ;;
        arm64|aarch64) echo "arma-macos-arm64" ;;
        *) echo "Unsupported macOS architecture: $arch"; exit 1 ;;
      esac
      ;;
    *)
      echo "Unsupported OS: $os"
      exit 1
      ;;
  esac
}

resolve_release_binary_url() {
  local target_tag asset_name
  if [[ -n "$TAG" ]]; then
    target_tag="$TAG"
  else
    target_tag="$(resolve_latest_tag)"
  fi

  asset_name="$(detect_asset_name)"
  echo "https://github.com/$REPO/releases/download/$target_tag/$asset_name"
}

write_wrapper() {
  cat > "$WRAPPER_BIN" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd "$APP_DIR"
exec "$TARGET_BIN" "\$@"
EOF
  chmod 755 "$WRAPPER_BIN"
}

install_systemd_service() {
  local unit_path="/etc/systemd/system/arma.service"
  cat > "$unit_path" <<EOF
[Unit]
Description=ARMA Prompt Guardrail Service
After=network.target

[Service]
Type=simple
WorkingDirectory=$APP_DIR
ExecStart=$WRAPPER_BIN start
ExecReload=$WRAPPER_BIN reload
Restart=always
RestartSec=2
User=$SERVICE_USER
Group=$SERVICE_GROUP
NoNewPrivileges=true
ProtectSystem=full
ProtectHome=true
PrivateTmp=true
ReadWritePaths=$APP_DIR

[Install]
WantedBy=multi-user.target
EOF
  systemctl daemon-reload
  systemctl enable arma.service
  echo "Installed systemd unit: $unit_path"
  echo "Run: sudo systemctl start arma"
}

ensure_service_account() {
  if id -u "$SERVICE_USER" >/dev/null 2>&1; then
    return
  fi

  if command -v useradd >/dev/null 2>&1; then
    useradd --system --home-dir "$APP_DIR" --shell /usr/sbin/nologin "$SERVICE_USER"
    return
  fi

  if command -v adduser >/dev/null 2>&1; then
    adduser --system --home "$APP_DIR" --shell /usr/sbin/nologin "$SERVICE_USER"
    return
  fi

  echo "Cannot create service user automatically. Please create user: $SERVICE_USER"
  exit 1
}

write_default_config() {
  cat > "$APP_DIR/config.yaml" <<EOF
server:
  host: 0.0.0.0
  port: 8080

logging:
  level: info
  path: ./logs/arma.log

filter_pack:
  dir: ./filter_packs
  profile: balanced
EOF
}

write_default_filter_packs() {
  mkdir -p "$APP_DIR/filter_packs"

  cat > "$APP_DIR/filter_packs/00-core.yaml" <<EOF
version: "1.0.0-core"
last_updated: "2026-02-22"

deny_keywords:
  - "ignore"
  - "ignore previous instructions"
  - "system prompt"
  - "developer message"
  - "시스템"
  - "무시"

deny_patterns:
  - "(?i)ignore\\s+all\\s+previous\\s+instructions"
  - "(?i)reveal\\s+.*\\s+prompt"

settings:
  sensitivity_score: 70
EOF

  cat > "$APP_DIR/filter_packs/99-custom.yaml" <<EOF
version: "1.0.0-custom"
last_updated: "2026-02-22"

allow_keywords:
  - "internal-approved-test"
  - "customer-whitelist-dummy"

settings:
  sensitivity_score: 75
EOF
}

install_binary() {
  local script_dir="$1"

  if [[ -z "$BINARY_URL" && ! -f "$script_dir/Cargo.toml" ]]; then
    BINARY_URL="$(resolve_release_binary_url)"
  fi

  if [[ $DRY_RUN -eq 1 ]]; then
    if [[ -n "$BINARY_URL" ]]; then
      echo "Dry run: remote binary install"
      echo "Resolved binary URL: $BINARY_URL"
    else
      echo "Dry run: local source build install"
      echo "Manifest path: $script_dir/Cargo.toml"
    fi
    echo "Install prefix: $PREFIX"
    echo "App dir: $APP_DIR"
    echo "Systemd: $WITH_SYSTEMD"
    return
  fi

  if [[ -n "$BINARY_URL" ]]; then
    require_cmd curl
    local tmp_bin
    tmp_bin="$(mktemp)"
    echo "Downloading binary from: $BINARY_URL"
    curl -fL "$BINARY_URL" -o "$tmp_bin"
    install -m 755 "$tmp_bin" "$TARGET_BIN"
    rm -f "$tmp_bin"
    return
  fi

  if [[ -f "$script_dir/Cargo.toml" ]]; then
    require_cmd cargo
    if [[ ! -x "$script_dir/target/release/arma" ]]; then
      echo "Building release binary..."
      cargo build --release --bin arma --manifest-path "$script_dir/Cargo.toml"
    fi
    install -m 755 "$script_dir/target/release/arma" "$TARGET_BIN"
    return
  fi

  echo "No local source tree detected and --binary-url not provided."
  echo "Use: sudo ./install.sh --binary-url <DIRECT_BINARY_URL>"
  exit 1
}

uninstall_systemd_service() {
  local unit_path="/etc/systemd/system/arma.service"
  if [[ -f "$unit_path" ]]; then
    systemctl disable arma.service >/dev/null 2>&1 || true
    systemctl stop arma.service >/dev/null 2>&1 || true
    rm -f "$unit_path"
    systemctl daemon-reload || true
  fi
}

if [[ "$MODE" == "uninstall" ]]; then
  if [[ $EUID -ne 0 ]]; then
    echo "Please run uninstall as root (sudo)."
    exit 1
  fi

  uninstall_systemd_service
  rm -f "$WRAPPER_BIN"
  rm -f "$TARGET_BIN"
  rmdir "$LIB_DIR" >/dev/null 2>&1 || true
  echo "Uninstalled ARMA binary and wrapper."
  echo "Config directory kept: $APP_DIR"
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ $DRY_RUN -eq 1 ]]; then
  install_binary "$SCRIPT_DIR"
  exit 0
fi

if [[ $EUID -ne 0 ]]; then
  echo "Please run install as root (sudo)."
  exit 1
fi

mkdir -p "$LIB_DIR" "$BIN_DIR" "$APP_DIR"

install_binary "$SCRIPT_DIR"
write_wrapper

if [[ ! -f "$APP_DIR/config.yaml" ]]; then
  if [[ -f "$SCRIPT_DIR/config.yaml" ]]; then
    install -m 644 "$SCRIPT_DIR/config.yaml" "$APP_DIR/config.yaml"
  else
    write_default_config
  fi
fi

if [[ ! -d "$APP_DIR/filter_packs" ]]; then
  if [[ -d "$SCRIPT_DIR/filter_packs" ]]; then
    mkdir -p "$APP_DIR/filter_packs"
    cp -a "$SCRIPT_DIR/filter_packs/." "$APP_DIR/filter_packs/"
  else
    write_default_filter_packs
  fi
fi

if [[ $WITH_SYSTEMD -eq 1 ]]; then
  require_cmd systemctl
  ensure_service_account
  chown -R "$SERVICE_USER:$SERVICE_GROUP" "$APP_DIR"
  install_systemd_service
fi

echo "Install complete."
echo "Binary: $TARGET_BIN"
echo "Wrapper: $WRAPPER_BIN"
echo "Config: $APP_DIR/config.yaml"
echo "Rules: $APP_DIR/filter_packs"
echo "Try: arma start"
