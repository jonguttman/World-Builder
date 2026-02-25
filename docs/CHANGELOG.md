# Changelog

## 0.3.0 - 2026-02-24
- FFI module (`sim-core/src/ffi.rs`) with C-compatible API for iOS bridge
- Opaque `SimHandle` type with cached flat arrays for zero-copy snapshot access
- Lifecycle functions: `pa_sim_create`, `pa_sim_destroy`
- Simulation control: `pa_sim_step`, `pa_sim_current_step`
- Snapshot accessors: temperatures, nutrients, moisture, population, ocean mask, biodiversity, species JSON
- Species management: `pa_sim_add_species_json`
- Intervention support: `pa_sim_apply_intervention_json`
- Save/load via `pa_sim_save_state`, `pa_sim_load_state`, `pa_free_bytes`
- Null-handle safety on all FFI functions
- 8 FFI integration tests (47 total tests passing)

## 0.2.0 - 2026-02-24
- Project scaffold with full design documents (11 spec docs)
- Rust sim-core crate initialized with all dependencies
- Core data structures: PlanetParams, WorldGrid, Tile, Species, Interventions
- Deterministic tick loop with ChaCha8 seeded RNG
- Climate model: temperature, greenhouse effect, albedo, seasonal variation, nutrient cycling
- Biosphere model: suitability calculation, logistic growth, carrying capacity, Holling Type II predation
- Mutation and speciation engine with epoch-based checks
- Intervention system: CO2, O2, cloud seeding, nutrient bloom, ice melt
- Level spec JSON format with objective evaluator (MicrobialStability, EcosystemStability, BiodiversityStability)
- Save/load with deterministic RNG state preservation via bincode
- Codex system with entry definitions, unlock triggers, and tracker
- Level 1 ("First Breath") JSON spec with full headless integration tests
- Swift bridge types mirroring Rust structures (SimTypes.swift)
- SwiftUI app shell: LevelSelectView with training/locked sections, PlanetView with time controls

## 0.1.0 - 2026-02-24
- Initial project setup
- Added README, .gitignore, and docs structure
