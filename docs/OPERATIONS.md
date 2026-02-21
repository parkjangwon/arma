# ARMA 운영환경 설치 가이드 (로컬 / Docker)

## 목차

- [1. 개요](#1-개요)
- [2. 로컬 설치/운영](#2-로컬-설치운영)
- [3. Docker 설치/운영](#3-docker-설치운영)
- [4. 운영 권장사항](#4-운영-권장사항)

## 1. 개요

이 문서는 운영 환경에서 ARMA를 설치하고 실행하는 절차를 설명합니다.

## 2. 로컬 설치/운영

### 2.1 빌드

```bash
cargo build --release
```

### 2.2 바이너리 배치 (선택)

```bash
sudo install -m 755 target/release/arma /usr/local/bin/arma
```

### 2.3 설정/룰셋 준비

- `config.yaml`
- `filter_packs/00-core.yaml`
- `filter_packs/99-custom.yaml`

### 2.4 기동/중지/리로드

```bash
arma start
arma reload
arma stop
```

### 2.5 운영 점검

```bash
curl -s http://127.0.0.1:8080/health
```

## 3. Docker 설치/운영

### 3.1 빌드

```bash
docker compose build
```

### 3.2 기동

```bash
docker compose up -d
```

### 3.3 상태 및 로그

```bash
docker compose ps
docker compose logs -f arma
```

### 3.4 무중단 룰셋 반영

- 호스트의 `./filter_packs/` 아래 YAML 수정
- 컨테이너 내부 `/app/filter_packs/`에 바인드 마운트되어 자동 감지

### 3.5 중지/정리

```bash
docker compose down
```

## 4. 운영 권장사항

- `logging.level`은 기본 `info`, 상세 분석 시에만 `debug` 사용
- Compose `json-file` 로깅 옵션(`10m`, `3`) 유지
- `server.host`는 컨테이너 환경에서 `0.0.0.0` 유지
- 룰셋 파일명은 `00-...`, `50-...`, `99-...` 패턴으로 우선순위 관리
