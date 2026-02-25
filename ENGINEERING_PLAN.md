# Engineering Plan (MVP)

## Architecture
- iOS app is thin client: UI, rendering, StoreKit, persistence
- sim-core is authoritative: all outcomes, objectives, codex triggers
- Bridge via FFI: Swift calls into sim-core with small, stable C ABI

## Determinism
- All floating math: prefer f32 with clamping and consistent ordering
- Avoid non-deterministic parallel reductions
- Snapshot serialization uses stable field ordering

## Performance
- Step in batches per frame
- Keep snapshots downsampled (64x32)

## Testing
- Golden tests: seed + interventions -> expected biodiversity curve hash
- Regression tests for every level spec
- UI tests for gating and purchases
