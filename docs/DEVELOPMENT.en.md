# ARMA Development Setup, Configuration, Run and Build Guide

## Table of Contents

- [1. Requirements](#1-requirements)
- [2. Prepare the project](#2-prepare-the-project)
- [3. Core configuration](#3-core-configuration)
- [4. Run locally](#4-run-locally)
- [5. Build locally](#5-build-locally)
- [6. Tests](#6-tests)
- [7. Run with Docker for development](#7-run-with-docker-for-development)

## 1. Requirements

- Rust stable
- Cargo
- Docker / Docker Compose (optional)
- Linux/macOS recommended

## 2. Prepare the project

```bash
git clone <your-repo-url>
cd arma
```

## 3. Core configuration

`config.yaml`

```yaml
server:
  host: 0.0.0.0
  port: 8080

logging:
  level: info
  path: ./logs/arma.log

filter_pack:
  dir: ./filter_packs
```

All `*.yaml` and `*.yml` files in `filter_packs/` are merged in ascending filename order.

## 4. Run locally

- Start server

```bash
cargo run --release -- start
```

- Stop server

```bash
cargo run --release -- stop
```

- Manual reload

```bash
cargo run --release -- reload
```

- Status (TUI)

```bash
cargo run --release -- status
```

## 5. Build locally

```bash
cargo build --release
```

Output: `target/release/arma`

## 6. Tests

```bash
cargo test
```

Stress test:

```bash
cargo run --release --bin stress
```

## 7. Run with Docker for development

- Build image

```bash
docker compose build
```

- Start container

```bash
docker compose up -d
```

- Tail logs

```bash
docker compose logs -f arma
```

- Stop and clean up

```bash
docker compose down
```
