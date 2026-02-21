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

## 빠른 시작

1) 로컬 실행

```bash
cargo run --release -- start
```

2) Docker 실행

```bash
docker compose up -d
```

3) 스트레스 테스트

```bash
cargo run --release --bin stress
```

실행 후 콘솔 결과와 함께 `ARMA_STRESS_TEST_REPORT_YYYYMMDD_HHMMSS.md` 리포트 파일이 생성됩니다.
