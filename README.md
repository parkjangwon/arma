# ARMA

<img width="3168" height="1344" alt="Gemini_Generated_Image_mmhfb6mmhfb6mmhf" src="https://github.com/user-attachments/assets/b23fd262-78fd-4bba-8451-d99fdb19f887" />

초고성능 AI 프롬프트 가드레일 엔진 (Rust).

ARMA는 LLM 호출 이전 단계에서 입력 프롬프트를 고속 검사해 우회 공격을 차단하는 경량 보안 게이트입니다.

## 이름과 발음

- 발음: **ARMA [ˈɑːr.mə]**, 한국어로는 **아르마**
- 네이밍 배경: **아르마딜로(armadillo)**에서 착안

작고 단단한 보호막이라는 이미지를 가져와, "화려함보다 신뢰 가능한 보호 기능"을 우선하는 프로젝트 철학을 이름에 담았습니다.

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
- **운영 친화성**: CLI 라이프사이클(start/stop/reload/status/update), JSON 로깅, Docker/Compose 지원

## 시스템 구성 요약

- API: `POST /v1/validate`, `GET /health`
- Rule Loader: `filter_packs/` 내 YAML 파일을 파일명 오름차순 병합 (`filter_pack.profile` 지정 시 해당 프로파일 파일만 선택)
- Engine 상태 공유: `Arc<RwLock<FilterEngine>>`
- 시그널 처리: SIGTERM graceful shutdown, SIGHUP manual reload

## 설치 가이드 (권장 순서)

### 1) 원격 설치 (가장 빠른 시작)

기본 동작은 **실행 권한 자동 판별**이다.

- `bash`로 실행(일반 계정): user 모드 설치
- `sudo bash`로 실행(root): system 모드 설치

일반 계정 설치(권장, sudo 없음)

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- --with-systemd
```

시스템 전역 설치(sudo)

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | sudo bash -s -- --with-systemd
```

설치 전에 대상 태그/에셋/명령만 확인하려면

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- --dry-run --with-systemd
```

삭제(클린 제거)

- 일반 계정 설치분(user 모드) 삭제:

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- uninstall
```

- 시스템 설치분(system 모드) 삭제:

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | sudo bash -s -- uninstall
```

주의: 삭제는 해당 모드의 ARMA 바이너리/서비스와 설정 디렉토리를 함께 제거한다.

삭제 드라이런(무삭제 미리보기):

```bash
# user 모드
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- uninstall --dry-run

# system 모드
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- uninstall --dry-run --scope system
```

고급 옵션(명시적 모드 지정): `--scope user|system`

업데이트(바이너리 + 필터팩 최신화)

- user 모드: `arma update` / `arma update --yes`
- system 모드: `sudo arma update` / `sudo arma update --yes`

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
