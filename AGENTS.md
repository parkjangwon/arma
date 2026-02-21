# ARMA Agent Operating Guide

## 1) Project Identity (Summary)
- Project: ARMA
- Purpose: Ultra-lightweight, ultra-high-performance AI prompt filtering guardrail in Rust.
- Delivery: Single executable binary and lightweight Docker container.
- Core philosophy:
  - Performance First: Minimize pre-LLM latency with aggressive optimization and zero-copy preference.
  - Zero-Impact: Internal failures or config syntax issues must not disrupt the customer main service (safe bypass).
  - Zero Maintenance: Operate with CLI, TUI, and file-based hot-reload configuration (no complex web UI).

## 10) Agent Code Guidelines (Summary)
1. No Panic: Never use `unwrap()` or `expect()`. Propagate all errors via `Result` and preserve bypass behavior.
2. Docs: Add `///` Rustdoc comments on all public APIs.
3. Efficiency: Minimize `.clone()`, prefer references (`&str`), and use async I/O via `tokio`.

## Enforcement
- Always read this file first before modifying code.
- Keep changes aligned with the ARMA philosophy and these coding rules.

## Team Memory: Why ARMA is Built This Way
- This project is intentionally optimized for **operational simplicity under pressure**.
- We prioritize **deterministic behavior** over clever abstractions.
- We treat ARMA as a **pre-LLM safety gate**, not as a business workflow engine.
- We optimize for the two hardest production moments:
  1) high QPS traffic spikes,
  2) live rule changes during active traffic.

## Architecture Intent (Do Not Break)
1. **Directory-based rule loading**
   - ARMA loads `filter_packs/*.yaml|*.yml` and merges by filename ascending order.
   - Vector fields (`deny_keywords`, `deny_patterns`, `allow_keywords`) are extended.
   - Scalar fields (`version`, `last_updated`, `sensitivity_score`) are overridden by the last file.
   - This ordering is a product feature for layered policy composition (`00-core`, `99-custom`).

2. **Hot-reload safety model**
   - Parse + validate outside lock.
   - Acquire write lock only for fast engine swap.
   - If parse/build fails, keep old engine and log warning.
   - Duplicate reload logs are suppressed via merged content digest comparison.

3. **Engine execution order (security contract)**
   - Normalize input.
   - Apply allow-list bypass check.
   - Run deny keyword scan (Aho-Corasick).
   - Run deny regex checks.
   - Keep this order unless security policy explicitly changes.

4. **Runtime resilience**
   - CLI lifecycle is first-class: `start/stop/restart/reload/status/manual`.
   - SIGHUP triggers manual reload path.
   - SIGTERM uses graceful shutdown.
   - Service should remain available even when rule/config update fails.

## Logging Direction
- Default operational mode is `info`.
- Request validation must emit concise one-line summary fields (action, reason, score, latency, keyword).
- Detailed watcher/loader internals belong to `debug`.
- `warn/error` should be reserved for meaningful operational faults.
- Logging changes must consider disk/I/O cost at high TPS.

## Vibe-Coding Maintenance Rules
- Keep edits **small, reversible, and explicit**.
- Preserve existing contracts before adding new features.
- Prefer extending current modules over introducing new frameworks.
- For production-impacting changes, always verify with:
  - `cargo check`
  - `cargo test`
  - run-path sanity (start + health + reload when relevant)

## Non-Negotiables for Next Agents
- Do not reintroduce single-file rule loading as default.
- Do not move heavy work into lock scopes.
- Do not weaken fail-safe bypass behavior on internal errors.
- Do not add noisy per-event logs at `info` in hot paths.
- Do not change API response semantics (`/v1/validate`, `/health`) without explicit versioning note.

## Preferred Change Strategy
1. Read `AGENTS.md` first.
2. Confirm if change touches hot path (validate, watcher, loader, server bind).
3. Keep behavior deterministic and observable.
4. Validate and document operational impact in `docs/` when behavior changes.
