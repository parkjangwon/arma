# ARMA Change Checklist (PR/배포 전 점검표)

`AGENTS.md`의 개발 철학과 운영 원칙을 실제 변경 작업에 적용하기 위한 체크리스트입니다.

## 1) 변경 전 이해

- [ ] `AGENTS.md`를 먼저 읽고 변경 범위를 정의했다.
- [ ] 변경이 핫패스(`validate`, `watcher`, `loader`, `server bind`)에 영향을 주는지 확인했다.
- [ ] 기존 API 계약(`/v1/validate`, `/health`) 변경 여부를 확인했다.
- [ ] 운영 영향(로그량, 성능, 무중단 리로드 안정성)을 사전에 가정했다.

## 2) 구현 원칙

- [ ] 락 안에서는 swap 등 최소 작업만 수행했다.
- [ ] 파싱/검증/빌드는 락 밖에서 수행했다.
- [ ] 단일 파일 룰 로딩으로 회귀하지 않았다(디렉토리 병합 유지).
- [ ] 실패 시 기존 엔진 유지(안전 우회) 동작을 유지했다.
- [ ] `info`에 과도한 고빈도 로그를 추가하지 않았다.

## 3) 코드 품질

- [ ] `unwrap()` / `expect()`를 새로 추가하지 않았다.
- [ ] Public API에는 필요한 Rustdoc(`///`)를 작성했다.
- [ ] 불필요한 clone, 불필요한 구조 변경을 피했다.
- [ ] 변경은 작고 되돌리기 쉽게 유지했다.

## 4) 검증

- [ ] `cargo check` 통과
- [ ] `cargo test` 통과
- [ ] 필요 시 실행 검증(`start`/`health`/`reload`) 수행
- [ ] 로깅 레벨(`info`/`debug`)에서 로그량 및 메시지 품질 확인

## 5) 운영 문서 반영

- [ ] 동작/설정 변경 시 `docs/` 문서를 함께 업데이트했다.
- [ ] 운영팀 영향이 있는 경우 `docs/OPERATIONS_RUNBOOK.md`를 업데이트했다.
- [ ] 벤더 연동 영향이 있는 경우 `docs/API_INTEGRATION.md`를 업데이트했다.

## 6) PR 본문 권장 템플릿

```text
## What
-

## Why
-

## Risk
-

## Validation
- cargo check
- cargo test
- runtime check (if applicable)

## Docs Updated
-
```
