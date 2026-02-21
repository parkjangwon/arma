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
- `filter_packs/10-profile-balanced.yaml` 또는 `filter_packs/10-profile-strict.yaml`

### 2.4 필터팩 프로파일 선택 가이드

`config.yaml`에서 아래처럼 `filter_pack.profile`을 지정합니다.

```yaml
filter_pack:
  dir: ./filter_packs
  profile: balanced # balanced | strict
```

프로파일 동작 규칙:
- `*-profile-<name>.yaml` 파일은 `profile` 값과 일치할 때만 병합됩니다.
- 예: `profile: strict`이면 `10-profile-strict.yaml`만 적용됩니다.
- `00-core.yaml`, `99-custom.yaml` 같은 공통 파일은 항상 병합됩니다.

운영 전환 절차(권장):
1) `config.yaml`의 `filter_pack.profile` 값을 변경
2) `arma reload` 실행 (또는 SIGHUP)
3) `curl -s http://127.0.0.1:8080/health`로 `filter_pack_version` 확인

### 2.5 기동/중지/리로드/업데이트

```bash
arma start
arma reload
arma stop
sudo arma update
```

`arma update` 동작:
- 최신 릴리즈 바이너리로 교체
- 필터팩 최신화 수행
- 필터팩 덮어쓰기 여부를 대화형으로 확인
- 완료 후 현재/최신 버전 정보를 출력

### 2.6 운영 점검

```bash
curl -s http://127.0.0.1:8080/health
```

`/health`에는 `filter_pack_version` 외에 `total_requests`, `block_rate`, `latency_p95_ms`, `top_block_reasons`가 포함되어 운영 상태를 빠르게 점검할 수 있습니다.

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
- 룰셋 파일명은 `00-core`, `99-custom` 기본 + 프로파일 팩(`10-profile-balanced.yaml`, `10-profile-strict.yaml`) 중 config의 `filter_pack.profile`에 맞는 파일 1개만 적용
- 도메인 팩(`50-finance.yaml.disabled`, `60-public-sector.yaml.disabled`, `70-ecommerce.yaml.disabled`)은 필요 시 활성화
- 고위험 탐지 팩은 `98-optional-high-risk.yaml.disabled`를 필요 시 `*.yaml`로 활성화
