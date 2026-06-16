# Parse Speedup — Baseline (fixed BEFORE any optimization)

Harness: `src/parser/src/bin/parse_bench.rs` — representative full parse (player props +
all events + ticks, `parse_ents=true`), release build (`lto=true, codegen-units=1`),
1 warm-up + 5 timed iterations, wall-clock min/median/max.

Machine/run: 2026-06-16, Windows. Numbers are median of 5.

| Demo | Size | ST median | MT median | MT speedup | ST throughput |
|------|------|-----------|-----------|------------|---------------|
| NaVi-vs-TheMongolz nuke | 429.8 MB | **8.448 s** | **1.723 s** | 4.9× | ~51 MB/s |
| test_demo (fixture) | 60.6 MB | 1.595 s | 0.471 s | 3.4× | ~38 MB/s |
| de_ancient HLTV | 230.6 MB | 3.495 s | 0.836 s | 4.2× | ~66 MB/s |

Primary reference for optimization work: **NaVi nuke** (largest, most representative).

## Targets (from PARSE_SPEEDUP_GOAL.md)
- ≥1.5× (target 2×) end-to-end vs these numbers, output byte-identical, all tests green.
- Reproduce: `cargo run --release --bin parse_bench -- <demo> 5 both` (or `st`/`mt`).

## Reproduce baseline
```
cargo build --release --bin parse_bench
./target/release/parse_bench "<nuke.dem>" 5 both
```
