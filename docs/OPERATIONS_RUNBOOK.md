# ARMA 운영 런북 (시스템 담당자용)

## 목차

- [1. 목적](#1-목적)
- [2. 서비스 라이프사이클](#2-서비스-라이프사이클)
- [3. config.yaml 운영 설정 가이드](#3-configyaml-운영-설정-가이드)
- [4. 필터팩 커스터마이징 절차](#4-필터팩-커스터마이징-절차)
- [5. 로그 분석 가이드](#5-로그-분석-가이드)
- [6. 이슈 트래킹 팁](#6-이슈-트래킹-팁)

## 1. 목적

이 문서는 운영 담당자가 ARMA 서비스를 안정적으로 기동/중지/재시작하고, 룰셋 변경 및 장애 분석을 빠르게 수행하기 위한 실무 지침입니다.

## 2. 서비스 라이프사이클

### 2.0 install.sh 기반 설치 (권장)

`install.sh`는 **ARMA 소스코드 루트 디렉토리에서 실행**하는 것을 기본으로 설계되어 있습니다.

```bash
cd /path/to/arma
sudo ./install.sh --with-systemd
```

이 스크립트는 소스 트리의 `Cargo.toml`, `config.yaml`, `filter_packs/`를 참조해 빌드/설치를 수행합니다.

소스 트리 없이 원라인 설치가 필요한 경우(예: 운영 서버 직접 배포), 릴리스 바이너리 URL을 사용해 설치할 수 있습니다.

```bash
curl -fsSL <INSTALL_SCRIPT_URL> | sudo bash -s -- --binary-url <DIRECT_BINARY_URL> --with-systemd
```

`--binary-url`이 없으면 로컬 소스(`Cargo.toml`) 기준 빌드 설치를 시도합니다.

GitHub 릴리즈 기반 원라인 설치(`install.sh`)도 지원합니다.

- 예시:

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | sudo bash -s -- --with-systemd
```

- 설치 전 드라이런:

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- --dry-run --with-systemd
```

- 설치 결과
  - 바이너리: `/usr/local/lib/arma/arma`
  - 전역 명령어 래퍼: `/usr/local/bin/arma`
  - 운영 설정/룰셋: `/etc/arma/config.yaml`, `/etc/arma/filter_packs/`
- systemd 옵션 사용 시
  - 전용 계정(`arma`)으로 서비스 실행
  - 하드닝 옵션 적용(`NoNewPrivileges`, `ProtectSystem` 등)

### 2.1 로컬/바이너리 운영

```bash
arma start
arma stop
arma restart
arma reload
arma status
```

### 2.2 Docker 운영

```bash
docker compose up -d
docker compose restart arma
docker compose down
docker compose logs -f arma
```

### 2.3 헬스체크

```bash
curl -s http://127.0.0.1:8080/health
```

정상 예시:

```json
{"status":"ok","filter_pack_version":"1.0.0-custom"}
```

## 3. config.yaml 운영 설정 가이드

핵심 설정 파일: `config.yaml`

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

### 3.1 `server.host` / `server.port`

- `server.host`
  - `0.0.0.0`: 외부/컨테이너 네트워크 수신 가능
  - `127.0.0.1`: 로컬 루프백만 수신
- `server.port`
  - API 리슨 포트 변경
  - 변경 후 클라이언트/헬스체크/리버스 프록시 설정도 같이 업데이트 필요

적용 방식:

- 현재 구현 기준, 포트/호스트 변경은 **프로세스 재기동**이 필요

### 3.2 `logging.level`

- `info` (권장 기본): 운영 지표/요약 로그 중심
- `debug` (일시적): watcher 이벤트/디렉토리 스캔 등 내부 동작 상세 출력
- `warn`/`error`: 장애 중심 로그만 확인하고 싶을 때 사용

적용 방식:

- 현재 구현 기준, 로그 레벨 변경은 **프로세스 재기동**이 필요

### 3.3 `filter_pack.dir`

- 룰셋 디렉토리 경로 변경
- 경로 변경 후 watcher가 새 디렉토리를 감시하며 YAML 병합 룰셋을 적용

적용 방식:

- 설정 파일 변경 후 `arma reload` 또는 파일 변경 감지로 반영 가능

### 3.4 변경 체크리스트 (권장)

변경 전:

1. 현재 상태 확인: `curl -s http://127.0.0.1:8080/health`
2. 기존 설정 백업: `cp config.yaml config.yaml.bak`
3. 변경 목적/영향 범위 기록

변경 후:

1. 적용 방식 수행(재기동 또는 `arma reload`)
2. 헬스체크 확인: `curl -s http://127.0.0.1:8080/health`
3. 로그 확인: `action`, `reason`, `latency_ms` 필드 정상 여부
4. 연동 시스템(프록시/벤더 클라이언트) 포트/주소 동기화 여부 확인

## 4. 필터팩 커스터마이징 절차

ARMA는 `filter_packs/` 디렉토리의 YAML을 파일명 오름차순으로 병합합니다.

- 권장 파일 전략
  - `00-core.yaml`: 전사 공통 보안 룰
  - `50-team.yaml`: 조직/서비스 팀 룰
  - `99-custom.yaml`: 고객별 예외/커스텀 룰

### 3.1 변경 규칙

- `deny_keywords`, `deny_patterns`, `allow_keywords`는 병합 시 누적됩니다.
- `version`, `last_updated`, `settings.sensitivity_score`는 마지막 파일 값이 우선합니다.
- 프로덕션 반영 전 문법 검증을 수행합니다.

### 3.2 반영 방법

1. 대상 YAML 수정
2. 저장 후 watcher 자동 반영 확인
3. 필요 시 수동 반영

```bash
arma reload
```

4. `/health`로 버전 확인

## 5. 로그 분석 가이드

### 4.1 INFO 로그 해석

프롬프트 검증 로그는 다음 필드를 포함합니다.

- `action`: PASS/BLOCK
- `latency_ms`: 요청 처리 시간
- `score`: 차단 점수
- `matched_keyword`: 매칭 키워드(`regex_pattern`, `none` 포함)
- `reason`: 최종 판단 사유

### 4.2 권장 필터링 예시

- BLOCK 이벤트만 보기
  - `action=BLOCK`
- 지연 상위 요청 찾기
  - `latency_ms` 기준 정렬
- 룰 과탐 여부 확인
  - 같은 `matched_keyword` 반복 빈도 확인

### 4.3 로깅 레벨 운영 정책

- 평시: `logging.level: info`
- 장애 분석 창구: `logging.level: debug` (단기)
- 장기 `debug` 상시 운영은 지양

## 6. 이슈 트래킹 팁

티켓 생성 시 아래 템플릿을 권장합니다.

- 발생 시각(타임존 포함)
- 영향 범위(요청률, 실패율, 고객/테넌트)
- 증상(예: BLOCK 급증, latency 증가, reload 실패)
- 근거 로그(1~3개 대표 이벤트)
- 직전 변경점(룰셋 파일명/커밋/배포 버전)
- 즉시 조치(룰 롤백, sensitivity 조정, reload 수행)
- 재발 방지 계획

운영 장애 시 핵심 우선순위:

1. 가용성 유지(서비스 연속성)
2. 오탐/미탐 위험 최소화
3. 변경 이력과 증적 보존
