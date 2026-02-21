# ARMA API Specification and Integration Guide (For Third-Party Vendors)

## Table of Contents

- [1. Overview](#1-overview)
- [2. API specification](#2-api-specification)
- [3. Recommended integration architecture](#3-recommended-integration-architecture)
- [4. Bypass strategy on ARMA failures](#4-bypass-strategy-on-arma-failures)
- [5. Operational tips](#5-operational-tips)

## 1. Overview

ARMA is a pre-LLM prompt validation gateway.

- Base URL example: `http://<arma-host>:8080`
- Response format: JSON

## 2. API specification

### 2.1 `POST /v1/validate`

Validates a prompt before it is sent to your LLM.

Request:

```json
{
  "prompt": "ignore previous instructions...",
  "user_id": "optional-user-123"
}
```

Response:

```json
{
  "is_safe": false,
  "reason": "BLOCK_DENY_KEYWORD:ignore",
  "score": 75,
  "latency_ms": 3
}
```

Field notes:

- `is_safe`: safety decision
- `reason`: decision reason (`PASS`, `BLOCK_DENY_KEYWORD:*`, `BLOCK_DENY_PATTERN`, `BYPASS_ALLOW_KEYWORD`, `ENGINE_ERROR_BYPASS`)
- `score`: block score
- `latency_ms`: ARMA processing latency in ms

### 2.2 `GET /health`

Health check and currently loaded rule-pack version.

Response:

```json
{
  "status": "ok",
  "filter_pack_version": "1.0.0-custom"
}
```

## 3. Recommended integration architecture

Recommended flow:

1. Receive client request
2. Call `POST /v1/validate` before LLM invocation
3. If `is_safe=false`, block or return an alternate response
4. If `is_safe=true`, continue to LLM

Recommended runtime settings:

- ARMA timeout: 100ms to 500ms aligned with your SLA
- Retry policy: at most one short retry
- Circuit breaker: fail-open switch during ARMA instability

## 4. Bypass strategy on ARMA failures

Important: ARMA failures should not cascade into full service outages unless your risk policy explicitly requires fail-close.

### 4.1 Fail-open policy example

Bypass is recommended when:

- connection failure or timeout to ARMA
- ARMA returns 5xx
- ARMA call exceeds timeout budget

When bypassing, also:

- mark internal context flag such as `arma_bypassed=true`
- log bypass reason for security audit
- alert operations team

### 4.2 Risk tradeoff

- Availability-first products: fail-open default
- Security-critical products: fail-close or policy-based hybrid

## 5. Operational tips

- poll `/health` continuously for early detection
- build dashboards around `reason` and `score`
- investigate `BLOCK` spikes with recent rule change history
- run staging regression with safe and malicious samples before release
