# ARMA

<img width="3168" height="1344" alt="Gemini_Generated_Image_mmhfb6mmhfb6mmhf" src="https://github.com/user-attachments/assets/072fd5f7-a867-4091-87b3-bd5774f46608" />

Ultra-high-performance AI prompt guardrail engine built in Rust.

ARMA is a lightweight security gateway that validates prompts before they reach your LLM, blocking prompt-injection and evasion patterns with low latency.

## Name and pronunciation

- Pronunciation: **ARMA [ˈɑːr.mə]** ("AR-ma")
- Naming origin: inspired by **armadillo**

The name reflects the project philosophy: compact, resilient protection over flashy surface features.

- Korean README: `README.md`
- Development docs (KO/EN): `docs/DEVELOPMENT.md`, `docs/DEVELOPMENT.en.md`
- Operations install guide (KO/EN): `docs/OPERATIONS.md`, `docs/OPERATIONS.en.md`
- Operations runbook (KO/EN): `docs/OPERATIONS_RUNBOOK.md`, `docs/OPERATIONS_RUNBOOK.en.md`
- API integration guide (KO/EN): `docs/API_INTEGRATION.md`, `docs/API_INTEGRATION.en.md`
- Documentation index: `docs/README.md`

## Highlights

- **High-speed filtering**: Aho-Corasick + Regex multi-layer checks
- **Normalization defense**: NFC + lowercase + whitespace/punctuation stripping
- **Zero-downtime hot reload**: directory-based rule merge with RwLock swap
- **Ops-ready runtime**: CLI lifecycle (start/stop/reload/status/update), JSON logging, Docker/Compose support

## Architecture at a glance

- API: `POST /v1/validate`, `GET /health`
- Rule Loader: merges YAML files in `filter_packs/` in filename ascending order (when `filter_pack.profile` is set, only matching profile files are merged)
- Shared engine state: `Arc<RwLock<FilterEngine>>`
- Signals: SIGTERM graceful shutdown, SIGHUP manual reload

## Quick start

1) Run locally

```bash
cargo run --release -- start
```

2) Run with Docker

```bash
docker compose up -d
```

3) Update installed runtime (binary + latest filter packs)

```bash
sudo arma update
# non-interactive mode with rule overwrite
sudo arma update --yes
```

4) Run stress test

```bash
cargo run --release --bin stress
```

After completion, the test prints metrics and writes a markdown report file named `ARMA_STRESS_TEST_REPORT_YYYYMMDD_HHMMSS.md`.
