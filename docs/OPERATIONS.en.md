# ARMA Operations Installation Guide (Local / Docker)

## Table of Contents

- [1. Overview](#1-overview)
- [2. Install mode comparison (user vs system)](#2-install-mode-comparison-user-vs-system)
- [3. Local installation and operations](#3-local-installation-and-operations)
- [4. Docker installation and operations](#4-docker-installation-and-operations)
- [5. Operational recommendations](#5-operational-recommendations)

## 1. Overview

This guide covers production-oriented installation and runtime steps for ARMA.


## 2. Install mode comparison (user vs system)

The installer auto-selects mode by privilege:

- run with `bash` (non-root) -> user mode
- run with `sudo bash` (root) -> system mode

### 2.1 Difference summary

- user mode
  - install path: `~/.local/lib/arma`, `~/.local/bin/arma`
  - config path: `~/.config/arma`
  - service: user session (Linux `systemctl --user`, macOS `~/Library/LaunchAgents`)
  - logs: user-owned path (`~/.local/state/arma` or config-relative path)
  - operations: start/stop/reload/logs without sudo

- system mode
  - install path: `/usr/local/lib/arma`, `/usr/local/bin/arma`
  - config path: `/etc/arma`
  - service: system-wide (Linux systemd system, macOS LaunchDaemons)
  - logs: system path (`/var/log` or config-relative path)
  - operations: sudo required

### 2.2 Install/uninstall commands

User-mode install (recommended):

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- --with-systemd
```

System-mode install:

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | sudo bash -s -- --with-systemd
```

User-mode uninstall:

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- uninstall
```

System-mode uninstall:

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | sudo bash -s -- uninstall
```

Advanced: force mode with `--scope user|system`.

## 3. Local installation and operations

### 3.1 Build

```bash
cargo build --release
```

### 3.2 Install binary (optional)

```bash
sudo install -m 755 target/release/arma /usr/local/bin/arma
```

### 3.3 Prepare configuration and rules

- `config.yaml`
- `filter_packs/00-core.yaml`
- `filter_packs/99-custom.yaml`

### 3.4 Start, reload, stop, update

```bash
# user mode
arma start
arma reload
arma stop
arma update

# system mode
sudo arma start
sudo arma reload
sudo arma stop
sudo arma update
```

`arma update` behavior:
- replaces runtime binary with latest release
- updates filter packs
- asks whether to overwrite local rule files
- prints current/latest version summary when done

### 3.5 Health check

```bash
curl -s http://127.0.0.1:8080/health
```

`/health` now includes runtime metrics such as `total_requests`, `block_rate`, `latency_p95_ms`, and `top_block_reasons` in addition to `filter_pack_version`.

## 4. Docker installation and operations

### 4.1 Build

```bash
docker compose build
```

### 4.2 Start

```bash
docker compose up -d
```

### 4.3 Status and logs

```bash
docker compose ps
docker compose logs -f arma
```

### 4.4 Zero-downtime rule updates

- Edit YAML files in host `./filter_packs/`
- They are bind-mounted to `/app/filter_packs/` in container and auto-reloaded

### 4.5 Stop and cleanup

```bash
docker compose down
```

## 5. Operational recommendations

- Keep `logging.level` at `info`; use `debug` only for investigation windows
- Keep Compose `json-file` rotation (`10m`, `3`) enabled
- Keep `server.host` at `0.0.0.0` in container environments
- Keep `00-core` and `99-custom` as baseline, and select one profile pack via `filter_pack.profile` (`10-profile-balanced.yaml` or `10-profile-strict.yaml`)
- Enable domain packs (`50-finance.yaml.disabled`, `60-public-sector.yaml.disabled`, `70-ecommerce.yaml.disabled`) only when needed
- Keep high-risk pack disabled by default (`98-optional-high-risk.yaml.disabled`) and enable only when needed
