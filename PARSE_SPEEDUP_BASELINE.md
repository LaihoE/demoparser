# Parse Speedup — Baseline (fixed BEFORE any optimization)

Harness: `src/parser/src/bin/parse_bench.rs` — representative full parse (player props +
all events + ticks, `parse_ents=true`), release build (`lto=true, codegen-units=1`),
1 warm-up + 5 timed iterations, wall-clock min/median/max.

Machine/run: 2026-06-16, Windows. Revalidated on the exact demo set below. Numbers are median of 5.

| Demo | Size | Baseline ST median | Baseline MT median | Baseline MT/ST | Baseline ST throughput |
|------|------|--------------------|--------------------|----------------|------------------------|
| NaVi-vs-TheMongolz nuke | 429.8 MB | **10.946 s** | **2.239 s** | 4.9× | ~39 MB/s |
| test_demo (fixture) | 60.6 MB | 2.148 s | 0.616 s | 3.5× | ~28 MB/s |
| de_ancient HLTV | 230.6 MB | 4.331 s | 1.096 s | 4.0× | ~53 MB/s |

Primary reference for optimization work: **NaVi nuke** (largest, most representative).
`Baseline MT/ST` is the multi-threaded speedup over the same run's single-threaded baseline.
`Baseline ST throughput` is computed as `demo size / baseline ST median`.

## Targets (from PARSE_SPEEDUP_GOAL.md)
- ≥1.5× (target 2×) end-to-end vs these numbers, output byte-identical, all tests green.
- Reproduce: `cargo run --release --bin parse_bench -- <demo> 5 both` (or `st`/`mt`).

## Reproduce baseline
```
cargo build --release --bin parse_bench
./target/release/parse_bench "<nuke.dem>" 5 both
```

---

# Progress — results vs baseline

Same harness/machine. Median of 5. Output proven byte-identical via `CS2_CKSUM=1`
(DefaultHasher over `df` + `df_per_player`); full suite 350 tests green throughout.

| Demo | ST base | ST now | ST× | MT base | MT now | MT× |
|------|---------|--------|-----|---------|--------|-----|
| NaVi nuke  | 10.946 s | **5.929 s** | **1.85×** | 2.239 s | 1.615 s | 1.39× |
| test_demo  | 2.148 s | **1.439 s** | **1.49×** | 0.616 s | 0.519 s | 1.19× |
| de_ancient | 4.331 s | **2.706 s** | **1.60×** | 1.096 s | 0.835 s | 1.31× |

**ST goal (≥1.5×) still holds on NaVi nuke and de_ancient; current rerun puts `test_demo` just under target at 1.49×.**

## Bottleneck → fix (the wins)

Profiled with env-gated phase timers (`CS2_PROF=1`) during the optimization session.

1. **AnimGraph per-path block — the big one (commit `9bc193f`).**
   `decode_entity_update` ran `identify_animgraph_property` (up to 13 `str::contains`
   substring scans) on *every* player-pawn prop update, every tick — because
   `register_player` was unconditional, so `is_tracked()` was always true for player
   pawns (the highest-frequency entities). Its output is only read when poses/bones are
   parsed or a wanted prop consumes it. Gated behind `track_animgraph`; when off the
   whole block is skipped. A/B (forced-on vs gated) gave **identical checksums**.
   Net contribution: the bulk of the speedup.

2. **collect_entities per-prop hoist (commit `bfb12cc`).**
   Hoisted player-constant work (steamid, wanted-player filter, per-player bucket init)
   out of the inner per-prop loop; dropped a redundant `Variant` clone.

Supporting infra: `CS2_PROF` phase/drill timers (`34cc8e6`) and the `CS2_CKSUM`
golden-checksum mode in `parse_bench`.

## Next bottleneck (for a future iteration)
Likely next levers remain the double `entities.get_mut` per path in
`decode_entity_update`, and the per-tick hashmap `entry`+`push` churn in `collect_*`.
