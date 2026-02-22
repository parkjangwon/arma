#!/usr/bin/env bash
set -euo pipefail

MODE="install"
WITH_SYSTEMD=0
SCOPE="auto"   # auto|system|user
PREFIX=""
APP_DIR=""
BIN_NAME="arma"
SERVICE_USER="arma"
SERVICE_GROUP="arma"
BINARY_URL=""
REPO="parkjangwon/arma"
TAG=""
DRY_RUN=0
UPDATE_RULES=0
OVERWRITE_RULES=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --with-systemd)
      WITH_SYSTEMD=1
      shift
      ;;
    --scope)
      SCOPE="$2"
      shift 2
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
    --update-rules)
      UPDATE_RULES=1
      shift
      ;;
    --overwrite-rules)
      OVERWRITE_RULES=1
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
      echo "Usage: ./install.sh [install|uninstall] [--scope auto|system|user] [--with-systemd] [--prefix PATH] [--app-dir PATH] [--service-user USER] [--binary-url URL] [--repo OWNER/REPO] [--tag TAG] [--dry-run] [--update-rules] [--overwrite-rules]"
      exit 1
      ;;
  esac
done

OS_NAME="$(uname -s | tr '[:upper:]' '[:lower:]')"

if [[ "$SCOPE" == "auto" ]]; then
  if [[ $EUID -eq 0 ]]; then
    SCOPE="system"
  else
    SCOPE="user"
  fi
fi

if [[ "$SCOPE" != "system" && "$SCOPE" != "user" ]]; then
  echo "Invalid --scope: $SCOPE (use auto|system|user)"
  exit 1
fi

if [[ -z "$PREFIX" ]]; then
  if [[ "$SCOPE" == "system" ]]; then
    PREFIX="/usr/local"
  else
    PREFIX="$HOME/.local"
  fi
fi

if [[ -z "$APP_DIR" ]]; then
  if [[ "$SCOPE" == "system" ]]; then
    APP_DIR="/etc/arma"
  else
    APP_DIR="$HOME/.config/arma"
  fi
fi

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

resolve_effective_tag() {
  if [[ -n "$TAG" ]]; then
    echo "$TAG"
  else
    resolve_latest_tag
  fi
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
  target_tag="$(resolve_effective_tag)"
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

install_systemd_user_service() {
  local unit_dir="$HOME/.config/systemd/user"
  local unit_path="$unit_dir/arma.service"
  mkdir -p "$unit_dir"
  cat > "$unit_path" <<EOF
[Unit]
Description=ARMA Prompt Guardrail Service (User)
After=default.target

[Service]
Type=simple
WorkingDirectory=$APP_DIR
ExecStart=$WRAPPER_BIN start
ExecReload=$WRAPPER_BIN reload
Restart=always
RestartSec=2

[Install]
WantedBy=default.target
EOF
  systemctl --user daemon-reload
  systemctl --user enable arma.service
  echo "Installed user systemd unit: $unit_path"
  echo "Run: systemctl --user start arma"
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

sync_filter_packs_from_source() {
  local source_dir="$1"
  local overwrite="$2"

  if [[ ! -d "$source_dir" ]]; then
    return 1
  fi

  mkdir -p "$APP_DIR/filter_packs"
  if [[ "$overwrite" -eq 1 ]]; then
    find "$APP_DIR/filter_packs" -maxdepth 1 -type f \( -name "*.yaml" -o -name "*.yml" \) -delete
  fi

  cp -a "$source_dir/." "$APP_DIR/filter_packs/"
  return 0
}

sync_filter_packs_from_release() {
  local target_tag="$1"
  local overwrite="$2"
  local archive_url="https://github.com/$REPO/archive/refs/tags/$target_tag.tar.gz"
  local workdir archive root_dir

  require_cmd curl
  require_cmd tar

  workdir="$(mktemp -d)"
  archive="$workdir/repo.tar.gz"

  if ! curl -fsSL "$archive_url" -o "$archive"; then
    rm -rf "$workdir"
    return 1
  fi

  if ! tar -xzf "$archive" -C "$workdir"; then
    rm -rf "$workdir"
    return 1
  fi

  root_dir="$(find "$workdir" -maxdepth 1 -type d -name "*" | grep -v "^$workdir$" | head -n 1)"
  if [[ -z "$root_dir" || ! -d "$root_dir/filter_packs" ]]; then
    rm -rf "$workdir"
    return 1
  fi

  sync_filter_packs_from_source "$root_dir/filter_packs" "$overwrite"
  rm -rf "$workdir"
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
    echo "Scope: $SCOPE"
    echo "Install prefix: $PREFIX"
    echo "App dir: $APP_DIR"
    echo "Service install: $WITH_SYSTEMD"
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

uninstall_systemd_user_service() {
  local unit_path="$HOME/.config/systemd/user/arma.service"
  systemctl --user disable arma.service >/dev/null 2>&1 || true
  systemctl --user stop arma.service >/dev/null 2>&1 || true
  if [[ -f "$unit_path" ]]; then
    rm -f "$unit_path"
  fi
  systemctl --user daemon-reload >/dev/null 2>&1 || true
}

install_launchd_service() {
  local plist_path="/Library/LaunchDaemons/org.arma.service.plist"
  cat > "$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>org.arma.service</string>
  <key>ProgramArguments</key>
  <array>
    <string>$WRAPPER_BIN</string>
    <string>start</string>
  </array>
  <key>WorkingDirectory</key>
  <string>$APP_DIR</string>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>/var/log/arma.out.log</string>
  <key>StandardErrorPath</key>
  <string>/var/log/arma.err.log</string>
</dict>
</plist>
EOF
  chown root:wheel "$plist_path"
  chmod 644 "$plist_path"
  launchctl bootout system "$plist_path" >/dev/null 2>&1 || true
  launchctl bootstrap system "$plist_path"
  launchctl enable system/org.arma.service >/dev/null 2>&1 || true
  echo "Installed launchd unit: $plist_path"
  echo "Run: sudo launchctl kickstart -k system/org.arma.service"
}

install_launchd_user_service() {
  local plist_dir="$HOME/Library/LaunchAgents"
  local plist_path="$plist_dir/org.arma.service.plist"
  mkdir -p "$plist_dir"
  cat > "$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>org.arma.service</string>
  <key>ProgramArguments</key>
  <array>
    <string>$WRAPPER_BIN</string>
    <string>start</string>
  </array>
  <key>WorkingDirectory</key>
  <string>$APP_DIR</string>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>$HOME/.local/state/arma/arma.out.log</string>
  <key>StandardErrorPath</key>
  <string>$HOME/.local/state/arma/arma.err.log</string>
</dict>
</plist>
EOF
  launchctl bootout "gui/$UID" "$plist_path" >/dev/null 2>&1 || true
  launchctl bootstrap "gui/$UID" "$plist_path"
  launchctl enable "gui/$UID/org.arma.service" >/dev/null 2>&1 || true
  echo "Installed user launchd unit: $plist_path"
  echo "Run: launchctl kickstart -k gui/$UID/org.arma.service"
}

uninstall_launchd_service() {
  local plist_path="/Library/LaunchDaemons/org.arma.service.plist"
  if [[ -f "$plist_path" ]]; then
    launchctl bootout system "$plist_path" >/dev/null 2>&1 || true
    rm -f "$plist_path"
  fi
}

uninstall_launchd_user_service() {
  local plist_path="$HOME/Library/LaunchAgents/org.arma.service.plist"
  if [[ -f "$plist_path" ]]; then
    launchctl bootout "gui/$UID" "$plist_path" >/dev/null 2>&1 || true
    rm -f "$plist_path"
  fi
}

resolve_script_dir() {
  local source_path="${BASH_SOURCE[0]-}"
  if [[ -n "$source_path" && "$source_path" != "bash" && -f "$source_path" ]]; then
    (cd "$(dirname "$source_path")" && pwd)
  else
    echo ""
  fi
}


if [[ "$MODE" == "uninstall" ]]; then
  if [[ "$SCOPE" == "system" && $EUID -ne 0 ]]; then
    echo "System scope uninstall requires root (sudo)."
    exit 1
  fi

  if [[ "$OS_NAME" == "darwin" ]]; then
    if [[ "$SCOPE" == "system" ]]; then
      uninstall_launchd_service
    else
      uninstall_launchd_user_service
    fi
  else
    if [[ "$SCOPE" == "system" ]]; then
      uninstall_systemd_service
    else
      uninstall_systemd_user_service
    fi
  fi
  rm -f "$WRAPPER_BIN"
  rm -f "$TARGET_BIN"
  rmdir "$LIB_DIR" >/dev/null 2>&1 || true
  rm -rf "$APP_DIR"
  echo "Uninstalled ARMA ($SCOPE scope) and removed config directory: $APP_DIR"
  exit 0
fi

SCRIPT_DIR="$(resolve_script_dir)"
EFFECTIVE_TAG="$(resolve_effective_tag)"
TAG="$EFFECTIVE_TAG"

if [[ $DRY_RUN -eq 1 ]]; then
  install_binary "$SCRIPT_DIR"
  echo "Resolved release tag: $EFFECTIVE_TAG"
  echo "Update rules mode: $UPDATE_RULES (overwrite=$OVERWRITE_RULES)"
  exit 0
fi

if [[ "$SCOPE" == "system" && $EUID -ne 0 ]]; then
  echo "System scope install requires root (sudo)."
  exit 1
fi

mkdir -p "$LIB_DIR" "$BIN_DIR" "$APP_DIR"
if [[ "$SCOPE" == "user" ]]; then
  mkdir -p "$HOME/.local/state/arma"
fi

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
  if ! sync_filter_packs_from_source "$SCRIPT_DIR/filter_packs" 1; then
    if ! sync_filter_packs_from_release "$EFFECTIVE_TAG" 1; then
      echo "Failed to initialize filter packs from source/release."
      exit 1
    fi
  fi
fi

if [[ $UPDATE_RULES -eq 1 ]]; then
  overwrite_mode=0
  if [[ $OVERWRITE_RULES -eq 1 ]]; then
    overwrite_mode=1
  fi

  if sync_filter_packs_from_source "$SCRIPT_DIR/filter_packs" "$overwrite_mode"; then
    echo "Filter packs updated from local source (overwrite=$overwrite_mode)."
  elif sync_filter_packs_from_release "$EFFECTIVE_TAG" "$overwrite_mode"; then
    echo "Filter packs updated from release $EFFECTIVE_TAG (overwrite=$overwrite_mode)."
  else
    echo "Failed to update filter packs from source/release."
    exit 1
  fi
fi

if [[ $WITH_SYSTEMD -eq 1 ]]; then
  if [[ "$OS_NAME" == "darwin" ]]; then
    require_cmd launchctl
    if [[ "$SCOPE" == "system" ]]; then
      install_launchd_service
    else
      install_launchd_user_service
    fi
  else
    require_cmd systemctl
    if [[ "$SCOPE" == "system" ]]; then
      ensure_service_account
      chown -R "$SERVICE_USER:$SERVICE_GROUP" "$APP_DIR"
      install_systemd_service
    else
      install_systemd_user_service
    fi
  fi
fi

echo "Install complete."
echo "Scope: $SCOPE"
echo "Binary: $TARGET_BIN"
echo "Wrapper: $WRAPPER_BIN"
echo "Config: $APP_DIR/config.yaml"
echo "Rules: $APP_DIR/filter_packs"
echo "Installed release tag: $EFFECTIVE_TAG"
echo "Try: arma start"
