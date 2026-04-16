# Clippy Lint Fixes — 2026-04-16

## Summary

Resolved all clippy lints in the 16 files modified on the `feature/topic-naming-and-ranges` branch. No lints from unmodified files were touched; those will be addressed in a separate branch.

## Files Changed

| File | Changes |
|------|---------|
| `build.rs` | Remove needless borrow on array literal passed to `Command::args` |
| `src/config_structs.rs` | Replace `format!("{}", x)` with direct `x` for `String` types |
| `src/consts.rs` | Suppress dead-code warnings on `APP_NAME` and `DASHBOARD_TAG_DESCRIPTION` public constants |
| `src/main.rs` | Remove `use std::fs`; suppress unused OTel imports; convert `match` to `if let`; fix let-and-return in `make_tracer`; collapse joinset check; remove dead `bar`/`tasks` bindings |
| `src/modules/controls/mod.rs` | Remove unused `get`, `post`, `Router`, `HashMap` imports; collapse nested `if` into combined condition; `for (_key,_)` → `for (_, _)` |
| `src/modules/dashboard/mod.rs` | Replace manual `Default` impl with `#[derive(Default)]`+`#[default]`; remove dead `grid_import_today`/`grid_export_today` accumulators; remove dead `query` variable; replace deprecated `DateTime::from_utc` with `from_naive_utc_and_offset`; remove unused `Timelike` import; replace `println!` with `info!` |
| `src/modules/mod.rs` | Remove `use sqlx::SqlitePool`; add `#[allow(clippy::upper_case_acronyms)]` on `RBAC`; add `#[allow(dead_code)]` on `Admin` variant and `data` method |
| `src/modules/settings/mod.rs` | Remove unused `load_config`, `use axum::routing::get`, `Router` imports |
| `src/monitored_point.rs` | Convert late-init `interval_checked` to inline expression; `unwrap_or_else(\|\| true)` → `unwrap_or(true)`; remove `mut` from non-mutated binding |
| `src/mqtt_poll.rs` | Remove `TryRecvError` and `Payload` unused imports; `len() > 0` → `is_empty()`; `match` → `if let`; remove dead inner `if let Payload::Config` block |
| `src/payload.rs` | Remove `num_traits::pow::Pow`; bind `val`/`point_data` via `if let` instead of `is_some()`+`unwrap()`; remove redundant `scaled_value: f64` declaration; convert late-init `stdev_checked` to inline; `deviations` `mut` removed via `unwrap_or`; `*int as i64` → `*int`; `stale.len() > 0` → `!is_empty()`; suppress `large_enum_variant` on `Payload` |
| `src/state.rs` | Remove unused `use sqlx::sqlite::SqlitePool` |
| `src/state_mgmt.rs` | Add `#[allow(dead_code)]` on `BitfieldHistory` and `AggregatedMeasurements`; `.get(0).clone()` → `.first()` |
| `src/sunspec_poll.rs` | Remove `mut` from non-mutated bindings; `.try_into().unwrap()` → `.into()` for `u32`→`i64`; unused destructure args `(addr, slave)` → `(_, _)`; remove needless borrows; `len() > 0` → `is_empty()`; `filter`+`collect` loop → `retain()` |
| `src/sunspec_unit.rs` | `format!("literal")` → `"literal".to_string()` (4×); collapse nested `if let Some(value) { if let ValueType::String(v)` → `if let Some(ValueType::String(v))`; add `#[allow(dead_code)]` on `points` field |

## Build Status

```
cargo build   → 0 errors, warnings only in non-branch files
cargo clippy  → 0 warnings in branch-modified files
cargo fmt     → all files formatted
```
