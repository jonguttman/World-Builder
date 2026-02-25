# Changelog

## 0.3.0 - 2026-02-24
- Implemented core data structures in sim-core/src/types.rs
  - PlanetParams with AtmosphereState and HydroState (Earth-like defaults)
  - WorldGrid with Tile map and coordinate helpers
  - Species and SpeciesTraits with TrophicLevel enum
  - Intervention system with InterventionKind variants and RegionTarget
  - TimeSpeed enum with steps_per_batch scaling
  - SimEvent enum for lifecycle and game events
- Added `pub use types::*` re-export in lib.rs
- Created integration test suite (5 tests) in tests/types_test.rs

## 0.2.0 - 2026-02-24
- Initialized Rust sim-core crate with Cargo.toml and dependencies
- Created placeholder modules: types, climate, biosphere, sim, level, codex, snapshot
- Added lib.rs with module declarations and main.rs entry point
- Added criterion benchmark scaffold

## 0.1.1 - 2026-02-24
- Project scaffold with full directory structure
- Added 11 design documents (codex, product brief, simulation spec, API spec, data models, UX flow, engineering plan, security, app store page, roadmap, agent prompts)
- Updated README with project overview

## 0.1.0 - 2026-02-24
- Initial project setup
- Added README, .gitignore, and docs structure
