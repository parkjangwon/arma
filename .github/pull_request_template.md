## What
-

## Why
-

## Risk
-

## Validation
- [ ] cargo check
- [ ] cargo test
- [ ] runtime check (if applicable)

## Docs Updated
-

## ARMA Change Checklist

- [ ] Read `AGENTS.md` and verified scope/intent
- [ ] Checked hot path impact (`validate`, `watcher`, `loader`, `server bind`)
- [ ] Preserved fail-safe behavior on internal/config errors
- [ ] Kept heavy parse/build work outside lock scopes
- [ ] Avoided noisy high-frequency `info` logs in hot paths
- [ ] No new `unwrap()` / `expect()` introduced
- [ ] Updated docs in `docs/` when behavior/config changed
