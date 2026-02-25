# Changelog

## 0.5.0 - 2026-02-24
### Sprint 3: Bundle Level JSON + LevelLoader
- Level 1 JSON (`level_01_first_breath.json`) bundled in iOS app resources
- `LevelLoader.swift` with `LevelConfig` Codable model and JSON decoding
- Helper methods `paramsJSON` and `objectiveJSON` for FFI bridge serialization
- `project.yml` updated with resources section for XcodeGen

## 0.4.0 - 2026-02-24
### Sprint 3: Level Play — Objective Evaluation FFI
- `pa_sim_snapshot_total_biomass` FFI function for querying total biomass across all tiles
- `pa_sim_evaluate_objective` FFI function for evaluating win/fail conditions from JSON objective
- `objective_result_cache` field on SimHandle for safe C string lifetime management
- C header updated with new objective evaluation declarations
- 2 new FFI tests (49 total Rust tests passing)

## 0.3.0 - 2026-02-24
### Sprint 2: iOS FFI Bridge & Visualization
- Rust FFI module with C-compatible API (18 extern functions, SimHandle with cached flat arrays)
- C header (`planet_architect.h`) for iOS bridging
- iOS cross-compilation build script (simulator + device)
- XcodeGen project spec with static library linking
- Swift `SimulationEngine` wrapper with type-safe FFI access
- `SimulationViewModel` with `@Observable` state management and simulation loop (~30fps)
- Canvas-based `GridRenderer` with temperature, nutrient, moisture, and population overlays
- Live `PlanetView` with overlay picker, time controls, and step/biodiversity display
- Tile inspector (tap grid to see tile details in sheet)
- 8 FFI tests (47 total Rust tests passing)
- iOS simulator build verified (BUILD SUCCEEDED)

## 0.2.0 - 2026-02-24
### Sprint 0+1: Foundations & Simulation Core
- Project scaffold with full design documents (11 spec docs)
- Rust sim-core crate with all dependencies
- Core data structures: PlanetParams, WorldGrid, Tile, Species, Interventions
- Deterministic tick loop with ChaCha8 seeded RNG
- Climate model: temperature, greenhouse effect, albedo, seasonal variation, nutrient cycling
- Biosphere model: suitability, logistic growth, carrying capacity, Holling Type II predation
- Mutation and speciation engine with epoch-based checks
- Intervention system: CO2, O2, cloud seeding, nutrient bloom, ice melt
- Level spec JSON format with objective evaluator
- Save/load with deterministic RNG state preservation
- Codex system with entry definitions, unlock triggers, and tracker
- Level 1 ("First Breath") JSON spec with headless integration tests
- Swift bridge types and SwiftUI app shell

## 0.1.0 - 2026-02-24
- Initial project setup
