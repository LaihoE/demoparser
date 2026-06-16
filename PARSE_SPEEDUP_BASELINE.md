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
(DefaultHasher over `df` + `df_per_player`); full suite 333 tests green.

| Demo | ST base | ST now | ST× | MT base | MT now | MT× |
|------|---------|--------|-----|---------|--------|-----|
| NaVi nuke  | 10.946 s | **4.940 s** | **2.22×** | 2.239 s | 1.123 s | 1.99× |
| test_demo  | 2.148 s | **1.015 s** | **2.12×** | 0.616 s | 0.459 s | 1.34× |
| de_ancient | 4.331 s | **1.959 s** | **2.21×** | 1.096 s | 0.545 s | 2.01× |

**ST goal (≥1.5×) holds on all three demos; target 2× is reached on all three ST runs.**

## Bottleneck → fix (the wins)

Profiled with env-gated phase timers (`CS2_PROF=1`) during the optimization session.

1. **Per-field event Vec allocation.**
   `decode_entity_update` called `listen_for_events()` for every updated field and
   received a fresh `Vec<GameEventInfo>` back, usually empty, then extended the packet
   event buffer. The function now appends directly into the existing `events_to_emit`
   buffer. Output checksum stayed identical.

2. **String dispatch in `create_custom_prop()`.**
   `collect_entities` previously matched custom props by `prop_name: &str` for every
   player and requested prop. The hot cases now dispatch on stable numeric `prop_info.id`;
   string matching remains only for the two full-name fallback props.

3. **Post-processing clone churn in `combine_outputs()`.**
   Single-threaded parsing has exactly one `SecondPassOutput`, but the old merge still
   cloned the full dataframe through `iter().map(|x| x.df.clone())`. ST now moves that
   output directly. The multi-threaded merge also moves per-segment data before combining
   instead of cloning whole segment outputs. `CS2_PROF=1` on NaVi showed
   `combine_outputs` drop from about `0.4 s` to `0.001 s` in ST.

Supporting infra: `CS2_PROF` phase timers and the `CS2_CKSUM` golden-checksum mode in
`parse_bench`.

## Next bottleneck (for a future iteration)
Likely next levers remain `decode_entity_update` (`find_field`, `get_propinfo`,
`bitreader.decode`, `insert_field`) and the per-tick hashmap `entry`+`push` churn in
`collect_*`.
