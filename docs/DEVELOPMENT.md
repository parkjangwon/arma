# ARMA 개발환경 구성 및 실행/빌드 가이드

## 목차

- [1. 요구사항](#1-요구사항)
- [2. 프로젝트 준비](#2-프로젝트-준비)
- [3. 주요 설정 파일](#3-주요-설정-파일)
- [4. 로컬 개발 실행](#4-로컬-개발-실행)
- [5. 로컬 빌드](#5-로컬-빌드)
- [6. 테스트](#6-테스트)
- [7. Docker 개발 실행](#7-docker-개발-실행)

## 1. 요구사항

- Rust stable
- Cargo
- Docker / Docker Compose (선택)
- Linux/macOS 권장

## 2. 프로젝트 준비

```bash
git clone <your-repo-url>
cd arma
```

## 3. 주요 설정 파일

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
  profile: balanced
```

`filter_packs/` 디렉토리의 `*.yaml`, `*.yml` 파일이 파일명 오름차순으로 병합됩니다.
`*-profile-<name>.yaml` 형식 파일은 `profile` 값과 일치할 때만 병합됩니다 (예: `10-profile-strict.yaml`).

## 4. 로컬 개발 실행

- 서버 시작

```bash
cargo run --release -- start
```

- 서버 중지

```bash
cargo run --release -- stop
```

- 수동 리로드

```bash
cargo run --release -- reload
```

- 상태(TUI)

```bash
cargo run --release -- status
```

- 설치 환경 업데이트(루트 권한 필요)

```bash
sudo arma update
```

## 5. 로컬 빌드

```bash
cargo build --release
```

산출물: `target/release/arma`

## 6. 테스트

```bash
cargo test
```

스트레스 테스트:

```bash
cargo run --release --bin stress
```

## 7. Docker 개발 실행

- 이미지 빌드

```bash
docker compose build
```

- 컨테이너 시작

```bash
docker compose up -d
```

- 로그 확인

```bash
docker compose logs -f arma
```

- 중지/정리

```bash
docker compose down
```
