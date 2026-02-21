# ARMA Operations Installation Guide (Local / Docker)

## Table of Contents

- [1. Overview](#1-overview)
- [2. Local installation and operations](#2-local-installation-and-operations)
- [3. Docker installation and operations](#3-docker-installation-and-operations)
- [4. Operational recommendations](#4-operational-recommendations)

## 1. Overview

This guide covers production-oriented installation and runtime steps for ARMA.

## 2. Local installation and operations

### 2.1 Build

```bash
cargo build --release
```

### 2.2 Install binary (optional)

```bash
sudo install -m 755 target/release/arma /usr/local/bin/arma
```

### 2.3 Prepare configuration and rules

- `config.yaml`
- `filter_packs/00-core.yaml`
- `filter_packs/99-custom.yaml`

### 2.4 Start, reload, stop

```bash
arma start
arma reload
arma stop
```

### 2.5 Health check

```bash
curl -s http://127.0.0.1:8080/health
```

## 3. Docker installation and operations

### 3.1 Build

```bash
docker compose build
```

### 3.2 Start

```bash
docker compose up -d
```

### 3.3 Status and logs

```bash
docker compose ps
docker compose logs -f arma
```

### 3.4 Zero-downtime rule updates

- Edit YAML files in host `./filter_packs/`
- They are bind-mounted to `/app/filter_packs/` in container and auto-reloaded

### 3.5 Stop and cleanup

```bash
docker compose down
```

## 4. Operational recommendations

- Keep `logging.level` at `info`; use `debug` only for investigation windows
- Keep Compose `json-file` rotation (`10m`, `3`) enabled
- Keep `server.host` at `0.0.0.0` in container environments
- Use filename priority conventions such as `00-...`, `50-...`, `99-...`
