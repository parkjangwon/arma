# ARMA

<img width="3168" height="1344" alt="Gemini_Generated_Image_mmhfb6mmhfb6mmhf" src="https://github.com/user-attachments/assets/b23fd262-78fd-4bba-8451-d99fdb19f887" />

초고성능 AI 프롬프트 가드레일 엔진 (Rust).

ARMA는 LLM 호출 이전 단계에서 입력 프롬프트를 고속 검사해 우회 공격을 차단하는 경량 보안 게이트입니다.

- English README: `README.en.md`
- 개발 문서(한글/영문): `docs/DEVELOPMENT.md`, `docs/DEVELOPMENT.en.md`
- 운영 설치 가이드(한글/영문): `docs/OPERATIONS.md`, `docs/OPERATIONS.en.md`
- 운영 런북(한글/영문): `docs/OPERATIONS_RUNBOOK.md`, `docs/OPERATIONS_RUNBOOK.en.md`
- API 연동 가이드(한글/영문): `docs/API_INTEGRATION.md`, `docs/API_INTEGRATION.en.md`
- 문서 인덱스: `docs/README.md`

## 핵심 특징

- **고성능 필터링**: Aho-Corasick + Regex 기반 다계층 검사
- **정규화 방어**: NFC/소문자화/공백·구두점 제거로 우회 입력 방어
- **무중단 Hot-reload**: 디렉토리 기반 룰셋 병합 후 RwLock 스왑
- **운영 친화성**: CLI 라이프사이클(start/stop/reload/status), JSON 로깅, Docker/Compose 지원

## 시스템 구성 요약

- API: `POST /v1/validate`, `GET /health`
- Rule Loader: `filter_packs/` 내 YAML 파일을 파일명 오름차순 병합
- Engine 상태 공유: `Arc<RwLock<FilterEngine>>`
- 시그널 처리: SIGTERM graceful shutdown, SIGHUP manual reload

## 설치 가이드 (권장 순서)

### 1) 원격 설치 (가장 빠른 시작)

퍼블릭 리포 기준으로 설치 스크립트를 바로 실행합니다.

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | sudo bash -s -- --with-systemd
```

설치 전에 대상 태그/에셋/명령만 확인하려면:

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- --dry-run --with-systemd
```

설치 후:

```bash
arma status
```

### 2) 로컬 소스 기반 설치

소스코드 루트에서 실행합니다.

```bash
sudo ./install.sh --with-systemd
```

개발/수동 실행만 필요한 경우:

```bash
cargo run --release -- start
```

### 3) 로컬 Docker 설치

```bash
docker compose build
docker compose up -d
docker compose ps
```

## 부하 테스트

```bash
cargo run --release --bin stress
```

실행 후 콘솔 결과와 함께 `ARMA_STRESS_TEST_REPORT_YYYYMMDD_HHMMSS.md` 리포트 파일이 생성됩니다.
