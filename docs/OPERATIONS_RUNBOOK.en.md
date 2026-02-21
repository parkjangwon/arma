# ARMA Operations Runbook (For System Operators)

## Table of Contents

- [1. Purpose](#1-purpose)
- [2. Service lifecycle](#2-service-lifecycle)
- [3. config.yaml operations settings](#3-configyaml-operations-settings)
- [4. Filter-pack customization process](#4-filter-pack-customization-process)
- [5. Log analysis guide](#5-log-analysis-guide)
- [6. Issue tracking tips](#6-issue-tracking-tips)

## 1. Purpose

This runbook helps operators safely start/stop/restart ARMA, customize rules, and investigate incidents quickly.

## 2. Service lifecycle

### 2.0 install.sh-based installation (recommended)

`install.sh` is designed to run from the **ARMA source repository root directory**.

```bash
cd /path/to/arma
sudo ./install.sh --with-systemd
```

The script reads `Cargo.toml`, `config.yaml`, and `filter_packs/` from the source tree to build and install ARMA.

If you need one-line installation without local source tree (for production servers), use release binary mode.

```bash
curl -fsSL <INSTALL_SCRIPT_URL> | sudo bash -s -- --binary-url <DIRECT_BINARY_URL> --with-systemd
```

`install.sh` also supports GitHub release-based one-line installation.

Filter-pack sync options:
- `--update-rules`: sync latest filter packs
- `--overwrite-rules`: sync with overwrite of existing YAML files

- Example:

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | sudo bash -s -- --with-systemd
```

- Dry run before install:

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- --dry-run --with-systemd
```

- Installation outputs
  - binary: `/usr/local/lib/arma/arma`
  - global wrapper command: `/usr/local/bin/arma`
  - runtime config/rules: `/etc/arma/config.yaml`, `/etc/arma/filter_packs/`
- With systemd option
  - service runs as dedicated user (`arma`)
  - hardening options enabled (`NoNewPrivileges`, `ProtectSystem`, etc.)

### 2.1 Local/binary operations

```bash
arma start
arma stop
arma restart
arma reload
arma status
sudo arma update
```

### 2.2 Docker operations

```bash
docker compose up -d
docker compose restart arma
docker compose down
docker compose logs -f arma
```

### 2.3 Health check

```bash
curl -s http://127.0.0.1:8080/health
```

Healthy response example:

```json
{"status":"ok","filter_pack_version":"1.0.0-custom"}
```

## 3. config.yaml operations settings

Core configuration file: `config.yaml`

```yaml
server:
  host: 0.0.0.0
  port: 8080

logging:
  level: info
  path: ./logs/arma.log

filter_pack:
  dir: ./filter_packs
  profile: balanced
```

### 3.1 `server.host` / `server.port`

- `server.host`
  - `0.0.0.0`: accepts external/container network traffic
  - `127.0.0.1`: loopback-only
- `server.port`
  - changes the API listening port
  - update clients/health checks/reverse proxy settings together

Apply behavior:

- with current implementation, host/port changes require a **process restart**

### 3.2 `logging.level`

- `info` (recommended default): operational summary logs
- `debug` (temporary): detailed internals like watcher events and directory scanning
- `warn`/`error`: incident-focused logs only

Apply behavior:

- with current implementation, log-level changes require a **process restart**

### 3.3 `filter_pack.dir`

- changes rule directory path
- watcher monitors the new directory and applies merged YAML rules

Apply behavior:

- can be applied via `arma reload` or watcher-triggered config update

### 3.4 Change checklist (recommended)

Before change:

1. Check current health: `curl -s http://127.0.0.1:8080/health`
2. Back up config: `cp config.yaml config.yaml.bak`
3. Record change purpose and expected impact

After change:

1. Apply via restart or `arma reload`
2. Validate health: `curl -s http://127.0.0.1:8080/health`
3. Validate logs for `action`, `reason`, and `latency_ms`
4. Confirm downstream systems (proxy/vendor clients) are updated with host/port changes

## 4. Filter-pack customization process

ARMA merges YAML files in `filter_packs/` in ascending filename order.

- Recommended file strategy
  - `00-core.yaml`: global baseline rules
  - `50-finance.yaml.disabled`: finance hardening rules (disabled by default)
  - `60-public-sector.yaml.disabled`: public-sector hardening rules (disabled by default)
  - `70-ecommerce.yaml.disabled`: e-commerce hardening rules (disabled by default)
  - `99-custom.yaml`: tenant-specific exceptions

- Optional high-risk pack
  - `98-optional-high-risk.yaml.disabled` is disabled by default
  - enable by renaming the file extension to `.yaml`
  - recommended for phased rollout because it may increase false positives

- Domain pack enable examples
  - `50-finance.yaml.disabled` -> `50-finance.yaml`
  - `60-public-sector.yaml.disabled` -> `60-public-sector.yaml`
  - `70-ecommerce.yaml.disabled` -> `70-ecommerce.yaml`

### 3.1 Merge semantics

- `deny_keywords`, `deny_patterns`, `allow_keywords` are accumulated.
- `version`, `last_updated`, `settings.sensitivity_score` are overridden by the last file.
- Validate YAML syntax before production rollout.

### 3.2 Apply changes

1. Edit target YAML
2. Save and verify watcher-based auto reload
3. Trigger manual reload if needed

```bash
arma reload
```

4. Check updated version via `/health`

## 5. Log analysis guide

### 4.1 INFO log fields

Validation logs include:

- `action`: PASS/BLOCK
- `latency_ms`: per-request latency
- `score`: block score
- `matched_keyword`: matched keyword (`regex_pattern`, `none` included)
- `reason`: final decision reason

### 4.2 Practical filtering examples

- View only blocked events
  - `action=BLOCK`
- Find latency outliers
  - sort by `latency_ms`
- Detect over-blocking candidates
  - repeated `matched_keyword` with high frequency

### 4.3 Logging level policy

- Normal operations: `logging.level: info`
- Incident window: `logging.level: debug` (temporary)
- Avoid always-on `debug` in production

## 6. Issue tracking tips

Recommended ticket template:

- occurrence time (with timezone)
- impact scope (request volume, failure ratio, tenant/customer)
- symptom (block spike, latency increase, reload failure)
- evidence logs (1 to 3 representative events)
- recent change context (rule filename/commit/deploy version)
- immediate mitigation (rule rollback, sensitivity tuning, manual reload)
- prevention actions

Incident priorities:

1. preserve service availability
2. minimize false positives/negatives
3. keep auditable change evidence
