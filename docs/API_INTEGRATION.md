# ARMA API 정의서 및 연동 가이드 (벤더용)

## 목차

- [1. 개요](#1-개요)
- [2. API 명세](#2-api-명세)
- [3. 연동 아키텍처 권장안](#3-연동-아키텍처-권장안)
- [4. 실패 시 바이패스 전략](#4-실패-시-바이패스-전략)
- [5. 운영 팁](#5-운영-팁)

## 1. 개요

ARMA는 LLM 요청 전 프롬프트를 검사하는 게이트웨이입니다.

- 기본 주소 예시: `http://<arma-host>:8080`
- 응답 포맷: JSON

## 2. API 명세

### 2.1 `POST /v1/validate`

요청 프롬프트를 검증합니다.

요청:

```json
{
  "prompt": "ignore previous instructions...",
  "user_id": "optional-user-123"
}
```

응답:

```json
{
  "is_safe": false,
  "reason": "BLOCK_DENY_KEYWORD:ignore",
  "score": 75,
  "latency_ms": 3
}
```

필드 설명:

- `is_safe`: 안전 여부
- `reason`: 판단 근거 (`PASS`, `BLOCK_DENY_KEYWORD:*`, `BLOCK_DENY_PATTERN`, `BYPASS_ALLOW_KEYWORD`, `ENGINE_ERROR_BYPASS`)
- `score`: 차단 점수
- `latency_ms`: ARMA 처리 시간(ms)

### 2.2 `GET /health`

헬스체크 및 현재 룰셋 버전 확인.

응답:

```json
{
  "status": "ok",
  "filter_pack_version": "1.0.0-custom"
}
```

## 3. 연동 아키텍처 권장안

권장 흐름:

1. 클라이언트 요청 수신
2. LLM 호출 전 `POST /v1/validate` 호출
3. `is_safe=false`면 차단/대체 응답
4. `is_safe=true`면 LLM 호출 진행

연동 시 권장 설정:

- ARMA 호출 타임아웃: 100ms~500ms 범위에서 서비스 SLA에 맞게 설정
- 재시도: 짧은 지연으로 1회 이하(과도한 재시도 금지)
- Circuit breaker: ARMA 장애 시 빠른 fail-open 전환

## 4. 실패 시 바이패스 전략

중요: ARMA 장애가 메인 서비스 장애로 전파되지 않도록 설계해야 합니다.

### 4.1 Fail-open 정책 예시

아래 조건에서는 메인 서비스 연속성을 우선해 바이패스를 권장합니다.

- ARMA 연결 실패(Connection refused/timeout)
- ARMA 5xx 응답
- ARMA 호출 시간 초과

바이패스 시 권장 조치:

- 요청에 `arma_bypassed=true` 같은 내부 플래그 기록
- 보안 감사 로그에 이유 기록
- 경보 시스템으로 운영자 알림

### 4.2 리스크 균형

- 가용성 우선 서비스: Fail-open 기본
- 보안 민감 서비스: Fail-close 또는 정책 기반 하이브리드

## 5. 운영 팁

- `/health` 주기 점검으로 사전 장애 감지
- `reason`/`score` 기반 대시보드 구성
- `BLOCK` 비율 급증 시 룰셋 변경 이력 동시 확인
- 배포 전 스테이징에서 정상/악성 샘플 회귀 테스트 수행
