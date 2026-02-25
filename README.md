# Planet Architect

Speculative evolution simulation game for iOS. Players shape planetary physics and observe emergent biospheres evolve over millions of years.

## Monetization
- Free: Levels 1–3 (training)
- $4.99: Levels 4–10 (Core Pack)
- $8.99: Levels 11–20 (Advanced Pack)
No ads, no subscription.

## Repo Structure
- `/ios` SwiftUI + Metal renderer, StoreKit2, UI, persistence
- `/sim-core` deterministic simulation engine (Rust)
- `/docs` specs, roadmap, plans

## MVP (v0.1)
- Levels 1–3
- Codex v1
- Deterministic sim loop + time controls
- Save/load
- StoreKit entitlements

## Build Notes
- iOS calls into sim-core via a thin bridge (FFI).
- Simulation must be deterministic: same seed + same interventions = same outcomes.
