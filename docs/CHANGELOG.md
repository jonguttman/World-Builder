# Changelog

## 0.6.0 - 2026-02-24
- Implemented biosphere microbe model in sim-core/src/biosphere.rs
  - suitability: gaussian temperature falloff, O2 need, toxin resistance, nutrient availability
  - carrying_capacity: trophic-level-scaled capacity modulated by suitability and nutrients
  - update_tile_populations: logistic growth with suitability, mortality, and predation pressure
  - compute_predation: Holling Type II functional response (Consumer eats Producer, Predator eats Consumer)
  - update_grid: applies population dynamics across entire WorldGrid
  - global_population: sums species population across all tiles
  - biodiversity_count: counts distinct surviving species
  - Extinction threshold at 0.5 removes tiny populations
- Created biosphere integration test suite (6 tests) in tests/biosphere_test.rs
  - Optimal suitability conditions
  - Wrong temperature reduces suitability
  - Population growth in good conditions
  - Population decline in bad conditions
  - Population bounded by carrying capacity
  - Extinction of tiny populations in lethal conditions

## 0.5.0 - 2026-02-24
- Added nutrient cycling model in sim-core/src/climate.rs
  - update_nutrients: volcanism input, ocean upwelling, biomass decay, moisture leaching
  - Nutrients clamped to [0.0, 1.0] range per tile per tick
- Integrated nutrient update into simulation tick loop (sim.rs)
- Created climate integration test suite (5 tests) in tests/climate_test.rs
  - Greenhouse effect: higher CO2 raises average temperature
  - Ice albedo: more ice lowers average temperature
  - Temperature bounds: extreme params stay within [-80, 80] range
  - Latitude gradient: equator warmer than poles
  - Nutrient volcanism: higher core heat increases nutrient levels

## 0.4.0 - 2026-02-24
- Implemented deterministic tick loop with seeded ChaCha8 RNG in sim-core/src/sim.rs
  - Simulation struct with new(), step(), tick(), snapshot(), and accessor methods
  - Reproducible world generation from any u64 seed
- Implemented basic climate model in sim-core/src/climate.rs
  - init_grid: latitude-based temperature, random elevation/moisture/nutrients, radiation shielding
  - update: per-tick temperature with seasonal variation, lapse rate from elevation, albedo cooling
- Implemented SimSnapshot in sim-core/src/snapshot.rs for serializable simulation state
- Added `pub use sim::Simulation` re-export in lib.rs
- Created determinism integration tests (3 tests) in tests/determinism_test.rs
  - Same seed produces identical results after 1000 steps
  - Different seeds produce measurably different worlds
  - Step counter advances correctly

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
