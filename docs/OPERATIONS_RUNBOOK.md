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

### 2.0 설치 모드 상세 (user / system)

ARMA는 설치 시 실행 권한을 기준으로 모드를 자동 선택한다.

- 일반 계정(`bash`) 실행: user 모드
- 관리자 계정(`sudo bash`) 실행: system 모드

운영 차이:
- user 모드
  - 서비스 제어: Linux `systemctl --user`, macOS `launchctl`(LaunchAgents)
  - 파일 경로: `~/.local`, `~/.config/arma`
  - 로그/상태 조회: sudo 불필요
- system 모드
  - 서비스 제어: Linux `systemctl`, macOS LaunchDaemons
  - 파일 경로: `/usr/local`, `/etc/arma`
  - 로그/상태 조회: sudo 필요

권장 정책:
- 개인 개발/단일 운영자: user 모드
- 서버 공용 운영/다중 사용자: system 모드

기본 설치 명령:

```bash
# user 모드 (권장)
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- --with-systemd

# system 모드
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | sudo bash -s -- --with-systemd
```

삭제 명령:

```bash
# user 모드 삭제
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- uninstall

# system 모드 삭제
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | sudo bash -s -- uninstall
```

삭제 전 미리보기(드라이런):

```bash
# user 모드
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- uninstall --dry-run

# system 모드
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- uninstall --dry-run --scope system
```


### 2.1 install.sh 설치/운영 절차 (상세)

소스 트리에서 직접 설치할 수도 있고, 원라인 설치도 가능하다.

```bash
cd /path/to/arma
# 일반 계정(user 모드)
./install.sh --with-systemd

# 시스템 전역(system 모드)
sudo ./install.sh --with-systemd
```

원라인 설치:

```bash
# user 모드
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- --with-systemd

# system 모드
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | sudo bash -s -- --with-systemd
```

드라이런:

```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/arma/main/install.sh | bash -s -- --dry-run --with-systemd
```

모드별 대표 경로:
- user 모드: `~/.local/lib/arma`, `~/.local/bin/arma`, `~/.config/arma`
- system 모드: `/usr/local/lib/arma`, `/usr/local/bin/arma`, `/etc/arma`

모드별 운영 명령:

```bash
# user 모드
arma start
arma stop
arma restart
arma reload
arma status
arma update

# system 모드
sudo arma start
sudo arma stop
sudo arma restart
sudo arma reload
sudo arma status
sudo arma update
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
  profile: balanced
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
  - `50-finance.yaml.disabled`: 금융 도메인 강화 룰(기본 비활성)
  - `60-public-sector.yaml.disabled`: 공공/행정 도메인 강화 룰(기본 비활성)
  - `70-ecommerce.yaml.disabled`: 커머스 도메인 강화 룰(기본 비활성)
  - `99-custom.yaml`: 고객별 예외/커스텀 룰

- 선택적 고위험 팩
  - `98-optional-high-risk.yaml.disabled`는 기본 비활성
  - 활성화하려면 파일명을 `.yaml`로 변경
  - 고위험 탐지 강화 대신 오탐 증가 가능성이 있으므로 단계적 적용 권장

- 도메인 팩 활성화 예시
  - `50-finance.yaml.disabled` -> `50-finance.yaml`
  - `60-public-sector.yaml.disabled` -> `60-public-sector.yaml`
  - `70-ecommerce.yaml.disabled` -> `70-ecommerce.yaml`

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
